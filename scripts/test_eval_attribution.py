import unittest

from scripts.eval_attribution import UciEngine, parse_exact_score


class ParseExactScoreTest(unittest.TestCase):
    def test_new_game_waits_for_engine_reset(self):
        engine = object.__new__(UciEngine)
        events = []

        def send(command):
            events.append(("send", command))

        def read_until(done):
            events.append(("read", done("readyok")))
            return ["readyok"]

        engine.send = send
        engine.read_until = read_until

        engine.new_game()

        self.assertEqual(
            events,
            [("send", "ucinewgame"), ("send", "isready"), ("read", True)],
        )

    def test_accepts_exact_score_at_requested_node_count(self):
        lines = [
            "info depth 12 score cp 23 nodes 200000 pv e2e4",
            "bestmove e2e4",
        ]
        self.assertEqual(parse_exact_score(lines, True, nodes=200000), 23)

    def test_rejects_exact_score_before_final_bounded_node_line(self):
        lines = [
            "info depth 12 score cp 23 nodes 160816 pv e2e4",
            "info depth 13 score cp 31 lowerbound nodes 200004 pv e2e4",
            "bestmove e2e4",
        ]
        with self.assertRaisesRegex(RuntimeError, "final search score is bounded"):
            parse_exact_score(lines, True, nodes=200000)

    def test_rejects_sufficient_exact_score_followed_by_bounded_score(self):
        lines = [
            "info depth 12 score cp 23 nodes 200000 pv e2e4",
            "info depth 13 score cp 31 lowerbound nodes 200004 pv e2e4",
            "bestmove e2e4",
        ]
        with self.assertRaisesRegex(RuntimeError, "final search score is bounded"):
            parse_exact_score(lines, True, nodes=200000)

    def test_accepts_exact_score_at_requested_depth(self):
        lines = [
            "info depth 14 score cp -17 nodes 84000 pv e7e5",
            "bestmove e7e5",
        ]
        self.assertEqual(parse_exact_score(lines, False, depth=14), 17)

    def test_rejects_bounded_requested_depth(self):
        lines = [
            "info depth 13 score cp -12 nodes 60000 pv e7e5",
            "info depth 14 score cp -17 upperbound nodes 84000 pv e7e5",
            "bestmove e7e5",
        ]
        with self.assertRaisesRegex(RuntimeError, "final search score is bounded"):
            parse_exact_score(lines, False, depth=14)

    def test_rejects_exact_score_beyond_requested_depth(self):
        lines = [
            "info depth 15 score cp -17 nodes 84000 pv e7e5",
            "bestmove e7e5",
        ]
        with self.assertRaisesRegex(RuntimeError, "expected exactly 14"):
            parse_exact_score(lines, False, depth=14)


if __name__ == "__main__":
    unittest.main()
