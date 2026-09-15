import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import compare_search_speed as speed


class SearchSpeedValidation(unittest.TestCase):
    def test_rejects_malformed_fen_before_launching_engine(self):
        bad = '6k1/6p1/6Pp/pppPp2P/1P1Ep3/2K5/8/8 b - - 0 1'
        with patch.object(speed, 'Engine') as engine:
            with self.assertRaises(ValueError):
                speed.search(Path('unused'), bad, 100000)
            engine.assert_not_called()
        self.assertFalse(speed.valid_fen('7k/8/8/8/8/8/8/K5 w - - 0 1'))
        self.assertFalse(speed.valid_fen('7k/8/8/8/8/8/8/8 w - - 0 1'))

    def test_engine_rejection_never_starts_search(self):
        with patch.object(speed, 'Engine') as factory:
            engine = factory.return_value
            engine.until.return_value = ['Error: Invalid FEN', 'readyok']
            with self.assertRaises(RuntimeError):
                speed.search(Path('unused'), '7k/8/8/8/8/8/8/K7 w - - 0 1', 100000)
            self.assertFalse(any(call.args[0].startswith('go ') for call in engine.send.call_args_list))
            engine.close.assert_called_once()

    def test_sample_has_eighty_valid_distinct_positions(self):
        fens = speed.sample_fens(Path(__file__).resolve().parents[1])
        self.assertEqual(len(fens), 80)
        self.assertTrue(all(speed.valid_fen(fen) for fen in fens))
        self.assertEqual(len({' '.join(fen.split()[:4]) for fen in fens}), 80)


if __name__ == '__main__':
    unittest.main()
