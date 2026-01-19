"""
SPSA Master Controller

Orchestrates the SPSA parameter tuning process:
1. Reads current parameters from params.toml
2. Generates perturbed parameter sets (plus/minus)
3. Builds engine binaries
4. Creates iteration record in database
5. Waits for workers to complete games
6. Calculates gradient and updates parameters
7. Repeats

Usage:
    cd rusty-rival
    python -m spsa.master
"""

import json
import math
import os
import random
import sys
import time
from datetime import datetime
from pathlib import Path

# Add chess-compete to path for database access
SPSA_DIR = Path(__file__).parent
RUSTY_RIVAL_DIR = SPSA_DIR.parent
CHESS_COMPETE_DIR = RUSTY_RIVAL_DIR.parent / 'chess-compete'
sys.path.insert(0, str(CHESS_COMPETE_DIR))

# Now we can import from chess-compete
from dotenv import load_dotenv
load_dotenv(CHESS_COMPETE_DIR / '.env')

try:
    import tomllib
except ImportError:
    import tomli as tomllib

from spsa.build import build_spsa_engines


def load_params() -> dict:
    """
    Load parameters from params.toml.
    Returns dict of {param_name: {'value': X, 'min': Y, 'max': Z, 'step': S}}
    """
    params_file = SPSA_DIR / 'params.toml'
    with open(params_file, 'rb') as f:
        return tomllib.load(f)


def save_params(params: dict):
    """
    Save parameters to params.toml.
    Preserves comments by reading original and only updating values.
    """
    params_file = SPSA_DIR / 'params.toml'

    # Read original file
    content = params_file.read_text()

    # Update each parameter's value
    for name, cfg in params.items():
        # Find and replace the value line for this parameter
        # Pattern: value = <number>
        import re
        pattern = rf'(\[{re.escape(name)}\][^\[]*value\s*=\s*)[\d.]+'
        replacement = rf'\g<1>{cfg["value"]}'
        content = re.sub(pattern, replacement, content, flags=re.DOTALL)

    params_file.write_text(content)


def load_config() -> dict:
    """Load configuration from config.toml."""
    config_file = SPSA_DIR / 'config.toml'
    with open(config_file, 'rb') as f:
        return tomllib.load(f)


def generate_perturbations(params: dict, c_k: float) -> tuple[dict, dict, dict]:
    """
    Generate perturbed parameter sets for SPSA.

    Args:
        params: Current parameter values with min/max/step
        c_k: Perturbation coefficient for this iteration

    Returns:
        (plus_params, minus_params, signs)
        - plus_params: dict of {param_name: value} for θ + c_k * Δ * step
        - minus_params: dict of {param_name: value} for θ - c_k * Δ * step
        - signs: dict of {param_name: +1 or -1} (the Δ values)
    """
    plus_params = {}
    minus_params = {}
    signs = {}

    for name, cfg in params.items():
        value = cfg['value']
        step = cfg['step']
        min_val = cfg['min']
        max_val = cfg['max']

        # Random sign: +1 or -1 (Bernoulli ±1)
        sign = random.choice([-1, 1])
        signs[name] = sign

        # Perturbation amount
        delta = c_k * step * sign

        # Apply with bounds
        plus_val = max(min_val, min(max_val, value + delta))
        minus_val = max(min_val, min(max_val, value - delta))

        plus_params[name] = plus_val
        minus_params[name] = minus_val

    return plus_params, minus_params, signs


def create_iteration(iteration_number: int, plus_path: str, minus_path: str,
                     config: dict, base_params: dict, plus_params: dict,
                     minus_params: dict, signs: dict) -> int:
    """
    Create a new SPSA iteration record in the database.

    Returns the iteration ID.
    """
    from web.app import create_app
    from web.database import db
    from web.models import SpsaIteration

    app = create_app()
    with app.app_context():
        iteration = SpsaIteration(
            iteration_number=iteration_number,
            plus_engine_path=plus_path,
            minus_engine_path=minus_path,
            timelow_ms=int(config['time_control']['timelow'] * 1000),
            timehigh_ms=int(config['time_control']['timehigh'] * 1000),
            target_games=config['games']['games_per_iteration'],
            status='pending',
            base_parameters={k: v['value'] for k, v in base_params.items()},
            plus_parameters=plus_params,
            minus_parameters=minus_params,
            perturbation_signs=signs,
        )
        db.session.add(iteration)
        db.session.commit()
        return iteration.id


def wait_for_completion(iteration_id: int, poll_interval: int = 30) -> dict:
    """
    Wait for workers to complete games for an iteration.

    Returns the completed iteration data.
    """
    from web.app import create_app
    from web.models import SpsaIteration

    while True:
        app = create_app()
        with app.app_context():
            iteration = SpsaIteration.query.get(iteration_id)
            if not iteration:
                raise RuntimeError(f"Iteration {iteration_id} not found!")

            progress = f"{iteration.games_played}/{iteration.target_games}"
            plus_score = iteration.plus_wins + iteration.draws * 0.5
            total = iteration.games_played or 1
            pct = plus_score / total * 100

            print(f"\r  Progress: {progress} games | Plus: +{iteration.plus_wins} -{iteration.minus_wins} ={iteration.draws} ({pct:.1f}%)", end='', flush=True)

            if iteration.games_played >= iteration.target_games:
                print()  # Newline after progress
                return {
                    'id': iteration.id,
                    'games_played': iteration.games_played,
                    'plus_wins': iteration.plus_wins,
                    'minus_wins': iteration.minus_wins,
                    'draws': iteration.draws,
                    'base_parameters': iteration.base_parameters,
                    'perturbation_signs': iteration.perturbation_signs,
                }

        time.sleep(poll_interval)


def calculate_gradient(results: dict, params: dict, c_k: float) -> tuple[dict, float]:
    """
    Calculate gradient estimate from game results.

    SPSA gradient estimate:
        g_k(θ) ≈ (L(θ + c_k*Δ) - L(θ - c_k*Δ)) / (2 * c_k * Δ)

    We use Elo difference as the loss function L.
    Since we want to maximize Elo (not minimize), gradient points toward improvement.

    Returns:
        (gradient, elo_diff)
    """
    total_games = results['games_played']
    plus_score = results['plus_wins'] + results['draws'] * 0.5

    # Score from plus engine's perspective (0 to 1)
    score = plus_score / total_games

    # Convert to Elo difference
    if score <= 0.001:
        elo_diff = -800.0
    elif score >= 0.999:
        elo_diff = 800.0
    else:
        elo_diff = -400 * math.log10(1 / score - 1)

    # Calculate gradient for each parameter
    signs = results['perturbation_signs']
    gradient = {}

    for name, cfg in params.items():
        if name in signs:
            sign = signs[name]
            step = cfg['step']
            # Gradient estimate: Δ_loss / (2 * c_k * step * sign)
            # Since sign is ±1, this simplifies to: elo_diff * sign / (2 * c_k * step)
            gradient[name] = elo_diff * sign / (2 * c_k * step)

    return gradient, elo_diff


def update_parameters(params: dict, gradient: dict, a_k: float) -> dict:
    """
    Update parameters using gradient estimate.

    θ_new = θ_old + a_k * gradient

    Respects min/max bounds for each parameter.
    """
    for name, cfg in params.items():
        if name in gradient:
            old_value = cfg['value']
            new_value = old_value + a_k * gradient[name]

            # Clamp to bounds
            new_value = max(cfg['min'], min(cfg['max'], new_value))

            cfg['value'] = new_value

    return params


def mark_iteration_complete(iteration_id: int, gradient: dict, elo_diff: float):
    """Mark iteration as complete and save results."""
    from web.app import create_app
    from web.database import db
    from web.models import SpsaIteration

    app = create_app()
    with app.app_context():
        iteration = SpsaIteration.query.get(iteration_id)
        iteration.status = 'complete'
        iteration.gradient_estimate = gradient
        iteration.elo_diff = elo_diff
        iteration.completed_at = datetime.utcnow()
        db.session.commit()


def get_last_iteration_number() -> int:
    """Get the last completed iteration number, or 0 if none."""
    from web.app import create_app
    from web.models import SpsaIteration

    app = create_app()
    with app.app_context():
        last = SpsaIteration.query.order_by(SpsaIteration.iteration_number.desc()).first()
        if last:
            return last.iteration_number
        return 0


def run_master():
    """Main SPSA master loop."""
    print(f"\n{'='*60}")
    print("SPSA MASTER CONTROLLER")
    print(f"{'='*60}")

    # Load configuration
    config = load_config()
    params = load_params()

    print(f"Parameters: {len(params)}")
    print(f"Games per iteration: {config['games']['games_per_iteration']}")
    print(f"Time control: {config['time_control']['timelow']}-{config['time_control']['timehigh']}s/move")
    print(f"Max iterations: {config['spsa']['max_iterations']}")
    print(f"{'='*60}")

    # SPSA hyperparameters
    a = config['spsa']['a']
    c = config['spsa']['c']
    A = config['spsa']['A']
    alpha = config['spsa']['alpha']
    gamma = config['spsa']['gamma']
    max_iterations = config['spsa']['max_iterations']
    poll_interval = config['database']['poll_interval_seconds']

    # Paths
    src_path = RUSTY_RIVAL_DIR
    output_base = Path(config['build']['engines_output_path'])
    if not output_base.is_absolute():
        output_base = SPSA_DIR / output_base

    # Get starting iteration
    start_iteration = get_last_iteration_number() + 1
    print(f"\nStarting from iteration {start_iteration}")

    print("\nCurrent parameter values:")
    for name, cfg in params.items():
        print(f"  {name}: {cfg['value']} (range: {cfg['min']}-{cfg['max']}, step: {cfg['step']})")

    # Main loop
    for k in range(start_iteration, max_iterations + 1):
        print(f"\n{'='*60}")
        print(f"ITERATION {k}/{max_iterations}")
        print(f"{'='*60}")

        # Calculate SPSA coefficients for this iteration
        a_k = a / ((k + A) ** alpha)
        c_k = c / (k ** gamma)
        print(f"Coefficients: a_k={a_k:.4f}, c_k={c_k:.4f}")

        # Generate perturbed parameters
        plus_params, minus_params, signs = generate_perturbations(params, c_k)

        print("\nPerturbations:")
        for name in params:
            base = params[name]['value']
            plus = plus_params[name]
            minus = minus_params[name]
            sign = '+' if signs[name] > 0 else '-'
            print(f"  {name}: {base:.2f} -> {sign} [{minus:.2f}, {plus:.2f}]")

        # Build engines
        print("\nBuilding engines...")
        try:
            plus_path, minus_path = build_spsa_engines(
                src_path, output_base,
                plus_params, minus_params,
                config['build']['plus_engine_name'],
                config['build']['minus_engine_name']
            )
        except Exception as e:
            print(f"ERROR: Build failed: {e}")
            print("Skipping iteration...")
            continue

        print(f"  Plus:  {plus_path}")
        print(f"  Minus: {minus_path}")

        # Create iteration record
        print("\nCreating iteration record...")
        iteration_id = create_iteration(
            k, str(plus_path), str(minus_path),
            config, params, plus_params, minus_params, signs
        )
        print(f"  Iteration ID: {iteration_id}")

        # Wait for workers
        print("\nWaiting for workers to complete games...")
        results = wait_for_completion(iteration_id, poll_interval)

        # Calculate gradient
        gradient, elo_diff = calculate_gradient(results, params, c_k)
        print(f"\nResults: Elo diff = {elo_diff:+.1f}")

        print("\nGradient estimates:")
        for name, g in gradient.items():
            print(f"  {name}: {g:+.4f}")

        # Update parameters
        print("\nUpdating parameters:")
        old_values = {name: cfg['value'] for name, cfg in params.items()}
        params = update_parameters(params, gradient, a_k)

        for name, cfg in params.items():
            old = old_values[name]
            new = cfg['value']
            delta = new - old
            print(f"  {name}: {old:.2f} -> {new:.2f} ({delta:+.4f})")

        # Save updated parameters
        save_params(params)
        print("\nSaved updated parameters to params.toml")

        # Mark iteration complete
        mark_iteration_complete(iteration_id, gradient, elo_diff)

    print(f"\n{'='*60}")
    print("SPSA TUNING COMPLETE")
    print(f"{'='*60}")
    print("\nFinal parameter values:")
    for name, cfg in params.items():
        print(f"  {name}: {cfg['value']:.2f}")


if __name__ == '__main__':
    try:
        run_master()
    except KeyboardInterrupt:
        print("\n\nMaster stopped by user.")
        sys.exit(0)
