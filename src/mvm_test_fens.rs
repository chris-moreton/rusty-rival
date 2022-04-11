pub fn get_test_fens() -> Vec<(&'static str, &'static str)> {
    vec![
        // 9. Rh6+ ..Kg7 10. Rh5 ..a5 11. Qc3 { Qb3 is worse, see below. Missing this is often the problem for this test. } ..Qxa4
        ("8/7R/ppqp1k2/4p3/PQn1P3/5P2/2P3r1/1KB5 b - - 8 40", "g2g1"),
        // avoid 41. Qb3 ..Rg1, 42. Qc3 { avoids Nd2 fork } ..Qc5 43. Ka2 { otherwise ..Rxc1, then if 44. Kxc1 ..Qg1+ 45. Qe1 ..Qxe1++ }
        ("8/7R/1pqp1k2/p3p3/PQn1P3/5P2/2P3r1/1KB5 w - - 0 41", "b4c3"),
        // Takes away g5 as an escape square for king, which traps the knight on c4 due to the threat of Qf7 mate
        // if 42 ..exf4, then Mate in 12. 43. Bb2 ..Nxb2 44. Qxb2+ ..Ke6
        //    45. Rh6+ ..Kd7 46. Qg7 ..Kd8 47. Rh8+ ..Qe8 48. Qg5+ ..Kc7 49. Rxe8 ..Rd1+
        //    50. Kb2 ..Rb1+ 51. Kxb1, etc...
        // if 42 ..Qxe4 43. Rc7 ..Qxc2+ 44. Qxc2 ..Rxc2 45. Kxc2
        // if 42 ..Rd1 43. fxe5 ..Ke6 44. Qh3!! mate in 7
        ("8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2Pr4/1KB5 w - - 2 42", "f3f4"),
        ("2r3k1/5pp1/p4n2/1p1Ppq2/2Pb1rBp/1P2R2P/PK1NQ1P1/3R4 w - - 7 32", "b2c1"),
        ("2r3k1/5pp1/p4n2/1p1Ppq2/2Pb1rBp/1P2R2P/P2NQ1P1/2KR4 b - - 8 32", "f6g4"),
        ("2r3k1/5pp1/p7/1p1Ppq2/2Pb1rnp/1P2R2P/P2NQ1P1/2KR4 w - - 0 33", "h3g4"),
        ("5r1k/8/2pn3p/1p1P4/pP2K3/P1N4P/8/3R4 w - - 1 44", "e4d4"),
        ("1r5k/8/7p/1pK5/pPn5/P1NR3P/8/8 b - - 2 47", "c4a3"),
        ("1B4k1/6p1/1p2n2p/p1p2p1P/P1P2P2/1P1K2P1/8/8 w - - 10 39", "b8a7"),
        ("8/7R/1p1p1k2/p3p3/P1n1qP2/1Q6/2Pr4/1KB5 w - - 0 43", "h7c7"),
        ("3r1k2/3q1pp1/p2b1n2/1p1Pp3/2P2r1p/4RB1P/PP1NQ1P1/1K2R3 b - c3 0 27", "b5c4"),
        ("3r1b1R/2k2pp1/Bn6/2n3P1/3Nbq2/1Q6/PPP5/1K1R4 w - - 9 30", "b3c3"),
        ("5Q2/4R1pk/p5q1/7p/1P2p2P/6PK/1r3P2/8 w - - 0 48", "e7e8"),
        ("8/3r4/6k1/1p1p3p/pP1PrR1P/P7/7K/5R2 b - - 2 45", "e4f4"),
        ("4k3/1pp2p2/2p3PK/4PP2/8/p1n3B1/2P5/8 b - - 0 31", "f7g6"),  // avoid 7.00+ blunder
        ("2r3k1/3q1pp1/p2b1n2/1p1Pp3/2P2r1p/1P2RB1P/P2NQ1P1/1K1R4 b - - 2 29", "d7f5"),
        ("3r1b1R/5pp1/Bnk5/2n3P1/3Nbq2/1Q6/PPP5/1K1R4 b - - 8 29", "d8d4"),
        ("5Q2/4R1pk/p5qp/8/1P2p2P/6P1/1r3P1K/8 w - - 0 47", "h2h3"),
        ("1r5k/8/7p/1p1K4/pPn5/P1NR3P/8/8 w - - 1 47", "c3b5"),
        ("6k1/5pp1/8/4KP1p/8/P3N1Pn/3p1P1P/2rR4 b - - 5 43", "c1c3"), // avoid 2.50+ blunder
        ("8/R1pkrp2/2p5/P3n1p1/6P1/6B1/2P2PK1/8 b - - 0 32", "e7e8"),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
        // ("", ""),
    ]
}

pub fn get_post_book_test_fens() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        // End-of-book-moves, Before first blunder, Blunder move, Best move after blunder
        (
            "3r1k2/3q1pp1/p2b1n2/1p1Pp3/5r1p/4RB1P/PPPNQ1P1/1K2R3 w - - 14 27",
            "3r1k2/3q1pp1/p2b1n2/1p1Pp3/5r1p/4RB1P/PPPNQ1P1/1K2R3 w - - 14 27",
            "c2c4",
            "d2e4",
        ),
        (
            "3r1k2/3q1pp1/p2b1n2/1p1Pp3/5r1p/4RB1P/PPPNQ1P1/1K2R3 w - - 14 27",
            "3r1k2/3q1pp1/p2b1n2/1p1Pp3/2P2r1p/4RB1P/PP1NQ1P1/1K2R3 b - - 0 27",
            "d8c8",
            "b5c4",
        ),
        (
            "3r1k2/3q1pp1/p2b1n2/1p1Pp3/5r1p/4RB1P/PPPNQ1P1/1K2R3 w - - 14 27",
            "2r3k1/3q1pp1/p2b1n2/1p1Pp3/2P2r1p/1P2RB1P/P2NQ1P1/1K2R3 w - - 1 29",
            "e1d1",
            "e1c1",
        ),
        (
            "2rqk2b/1p2pp2/p1np4/4n1p1/2bNP3/2N2P2/PPPQ1BP1/R3KB2 w Q - 2 17",
            "2r1k3/1pq1pp2/p2p4/8/2n1P3/1Q3P2/P1P2BP1/2K4R b - - 9 26",
            "e7e5",
            "e8d7",
        ),
        (
            "2rqk2b/1p2pp2/p1np4/4n1p1/2bNP3/2N2P2/PPPQ1BP1/R3KB2 w Q - 2 17",
            "8/7R/ppqp1k2/4p3/P1n1P3/1Q3P2/2P3r1/1KB5 w - - 7 40",
            "b3b4",
            "e8d7",
        ),
        (
            "2rqk2b/1p2pp2/p1np4/4n1p1/2bNP3/2N2P2/PPPQ1BP1/R3KB2 w Q - 2 17",
            "8/7R/1pqp1k2/p3p3/PQn1P3/5P2/2P3r1/1KB5 w - - 0 41",
            "b4b3",
            "g2g1",
        ),
        (
            "2rqk2b/1p2pp2/p1np4/4n1p1/2bNP3/2N2P2/PPPQ1BP1/R3KB2 w Q - 2 17",
            "8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2P3r1/1KB5 b - - 1 41",
            "g2d2",
            "g2g1",
        ), // Huge swing
        (
            "r1b1k2r/1pq2ppp/p3pn2/n2p4/1b2PB2/1NN2P2/PPPQ2PP/2KR1B1R b kq - 1 11",
            "5rk1/4qp1p/p3b1p1/1pr5/2P1QP2/1P1B4/P5PP/1K2R2R w - - 0 25",
            "b3b4",
            "g2g4",
        ),
        (
            "3rr2k/3n4/2pPpp1p/1p4p1/pP1P2PN/P1Nn3P/3R4/5RK1 w - g6 0 31",
            "5r1k/6n1/2p4p/1p6/pP1PK3/P1N4P/8/3R4 b - - 2 42",
            "g7e8",
            "g7e6",
        ),
        (
            "3rr2k/3n4/2pPpp1p/1p4p1/pP1P2PN/P1Nn3P/3R4/5RK1 w - g6 0 31",
            "1r5k/8/7p/1p1K4/pPn5/P1NR3P/8/8 w - - 1 47",
            "d5c5",
            "c3b5",
        ),
        ("", "", "", ""),
    ]
}
