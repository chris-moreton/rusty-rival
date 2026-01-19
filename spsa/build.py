"""
Engine building utilities for SPSA tuning.

Modifies engine_constants.rs with parameter values and compiles the engine.
"""

import os
import re
import shutil
import subprocess
from pathlib import Path


# Mapping from params.toml names to engine_constants.rs patterns
# Format: param_name -> (regex_pattern, replacement_template)
# The template uses {value} as placeholder
PARAM_MAPPINGS = {
    # Simple constants
    'beta_prune_margin_per_depth': (
        r'pub const BETA_PRUNE_MARGIN_PER_DEPTH: Score = \d+;',
        'pub const BETA_PRUNE_MARGIN_PER_DEPTH: Score = {value};'
    ),
    'beta_prune_max_depth': (
        r'pub const BETA_PRUNE_MAX_DEPTH: u8 = \d+;',
        'pub const BETA_PRUNE_MAX_DEPTH: u8 = {value};'
    ),
    'null_move_reduce_depth_base': (
        r'pub const NULL_MOVE_REDUCE_DEPTH_BASE: u8 = \d+;',
        'pub const NULL_MOVE_REDUCE_DEPTH_BASE: u8 = {value};'
    ),
    'null_move_min_depth': (
        r'pub const NULL_MOVE_MIN_DEPTH: u8 = \d+;',
        'pub const NULL_MOVE_MIN_DEPTH: u8 = {value};'
    ),
    'see_prune_margin': (
        r'pub const SEE_PRUNE_MARGIN: Score = \d+;',
        'pub const SEE_PRUNE_MARGIN: Score = {value};'
    ),
    'see_prune_max_depth': (
        r'pub const SEE_PRUNE_MAX_DEPTH: u8 = \d+;',
        'pub const SEE_PRUNE_MAX_DEPTH: u8 = {value};'
    ),
    'threat_extension_margin': (
        r'pub const THREAT_EXTENSION_MARGIN: Score = \d+;',
        'pub const THREAT_EXTENSION_MARGIN: Score = {value};'
    ),
}

# Array parameters need special handling
# These are computed from base + index * per_depth
ARRAY_PARAM_MAPPINGS = {
    'alpha_prune_margins': {
        'pattern': r'pub const ALPHA_PRUNE_MARGINS: \[Score; 8\] = \[[^\]]+\];',
        'template': 'pub const ALPHA_PRUNE_MARGINS: [Score; 8] = [{values}];',
        'size': 8,
        'base_param': 'alpha_prune_margin_base',
        'step_param': 'alpha_prune_margin_per_depth',
    },
    'lmp_move_thresholds': {
        'pattern': r'pub const LMP_MOVE_THRESHOLDS: \[u8; 4\] = \[[^\]]+\];',
        'template': 'pub const LMP_MOVE_THRESHOLDS: [u8; 4] = [{values}];',
        'params': ['0', 'lmp_threshold_depth1', 'lmp_threshold_depth2', 'lmp_threshold_depth3'],
    },
}


def read_engine_constants(src_path: Path) -> str:
    """Read the engine_constants.rs file."""
    constants_file = src_path / 'src' / 'engine_constants.rs'
    return constants_file.read_text()


def write_engine_constants(src_path: Path, content: str):
    """Write the engine_constants.rs file."""
    constants_file = src_path / 'src' / 'engine_constants.rs'
    constants_file.write_text(content)


def apply_parameters(content: str, params: dict) -> str:
    """
    Apply parameter values to engine_constants.rs content.

    Args:
        content: Current engine_constants.rs content
        params: Dict of {param_name: value}

    Returns:
        Modified content
    """
    # Apply simple constant mappings
    for param_name, (pattern, template) in PARAM_MAPPINGS.items():
        if param_name in params:
            value = int(round(params[param_name]))
            replacement = template.format(value=value)
            content = re.sub(pattern, replacement, content)

    # Apply ALPHA_PRUNE_MARGINS (computed from base + index * per_depth)
    if 'alpha_prune_margin_base' in params and 'alpha_prune_margin_per_depth' in params:
        base = int(round(params['alpha_prune_margin_base']))
        step = int(round(params['alpha_prune_margin_per_depth']))
        values = [base + i * step for i in range(8)]
        values_str = ', '.join(str(v) for v in values)

        mapping = ARRAY_PARAM_MAPPINGS['alpha_prune_margins']
        replacement = mapping['template'].format(values=values_str)
        content = re.sub(mapping['pattern'], replacement, content)

    # Apply LMP_MOVE_THRESHOLDS
    lmp_params = ['lmp_threshold_depth1', 'lmp_threshold_depth2', 'lmp_threshold_depth3']
    if any(p in params for p in lmp_params):
        values = [0]  # Index 0 is always 0
        for p in lmp_params:
            if p in params:
                values.append(int(round(params[p])))
            else:
                # Default values if not specified
                defaults = {'lmp_threshold_depth1': 8, 'lmp_threshold_depth2': 12, 'lmp_threshold_depth3': 16}
                values.append(defaults[p])
        values_str = ', '.join(str(v) for v in values)

        mapping = ARRAY_PARAM_MAPPINGS['lmp_move_thresholds']
        replacement = mapping['template'].format(values=values_str)
        content = re.sub(mapping['pattern'], replacement, content)

    return content


def build_engine(src_path: Path, output_path: Path, params: dict = None) -> bool:
    """
    Build the engine with optional parameter modifications.

    Args:
        src_path: Path to rusty-rival source
        output_path: Path to output the binary (directory)
        params: Optional dict of parameter values to apply

    Returns:
        True if build succeeded
    """
    # Backup original constants if we're modifying
    constants_file = src_path / 'src' / 'engine_constants.rs'
    original_content = None

    try:
        if params:
            # Read, modify, write
            original_content = read_engine_constants(src_path)
            modified_content = apply_parameters(original_content, params)
            write_engine_constants(src_path, modified_content)

        # Build with native optimizations
        env = os.environ.copy()
        env['RUSTFLAGS'] = '-C target-cpu=native'

        result = subprocess.run(
            ['cargo', 'build', '--release'],
            cwd=src_path,
            env=env,
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print(f"Build failed:\n{result.stderr}")
            return False

        # Copy binary to output location
        output_path.mkdir(parents=True, exist_ok=True)

        # Handle Windows vs Unix binary names
        if os.name == 'nt':
            binary_name = 'rusty-rival.exe'
        else:
            binary_name = 'rusty-rival'

        src_binary = src_path / 'target' / 'release' / binary_name
        dst_binary = output_path / binary_name

        shutil.copy2(src_binary, dst_binary)

        return True

    finally:
        # Restore original constants
        if original_content is not None:
            write_engine_constants(src_path, original_content)


def build_spsa_engines(src_path: Path, output_base: Path,
                       plus_params: dict, minus_params: dict,
                       plus_name: str = 'spsa-plus', minus_name: str = 'spsa-minus') -> tuple[Path, Path]:
    """
    Build both plus and minus perturbed engines for SPSA iteration.

    Args:
        src_path: Path to rusty-rival source
        output_base: Base path for engine output directories
        plus_params: Parameter values for plus engine
        minus_params: Parameter values for minus engine
        plus_name: Name for plus engine directory
        minus_name: Name for minus engine directory

    Returns:
        (plus_engine_path, minus_engine_path) - full paths to engine binaries
    """
    plus_dir = output_base / plus_name
    minus_dir = output_base / minus_name

    print(f"Building plus engine ({plus_name})...")
    if not build_engine(src_path, plus_dir, plus_params):
        raise RuntimeError("Failed to build plus engine")

    print(f"Building minus engine ({minus_name})...")
    if not build_engine(src_path, minus_dir, minus_params):
        raise RuntimeError("Failed to build minus engine")

    # Return paths to binaries
    binary_name = 'rusty-rival.exe' if os.name == 'nt' else 'rusty-rival'
    return plus_dir / binary_name, minus_dir / binary_name


if __name__ == '__main__':
    # Test: build with current params
    import tomllib

    spsa_dir = Path(__file__).parent
    src_path = spsa_dir.parent

    # Load params
    with open(spsa_dir / 'params.toml', 'rb') as f:
        params_config = tomllib.load(f)

    # Extract current values
    params = {name: cfg['value'] for name, cfg in params_config.items()}

    print(f"Parameters: {params}")
    print(f"Building engine with current parameters...")

    output_path = spsa_dir / 'test-build'
    if build_engine(src_path, output_path, params):
        print(f"Success! Binary at: {output_path}")
    else:
        print("Build failed!")
