use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc, ximgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::{Point, Point2f, Rect, Vector, ToInputArray, InputArray, count_non_zero, BORDER_DEFAULT, Size_, min_max_loc, subtract, no_array, pow, sum_elems};
use screenshots::Screen;
use std::{fs};
use std::ops::{Add, RangeInclusive};
use std::process::Command;
use std::thread::sleep;
use opencv::types::VectorOfVectorOfPoint;
use crate::bitboards::{A1_BIT, A8_BIT, bit, H1_BIT, H8_BIT, test_bit};
use crate::fen::{algebraic_move_from_move, get_position};
use crate::search::iterative_deepening;
use crate::types::{BLACK, default_search_state, WHITE};

const BOARD_IMAGE_WIDTH: i32 = 2932;
const RESIZED_BOARD_IMAGE_WIDTH: i32 = 1024;
const CHESSBOARD_X: i32 = 1322;
const CHESSBOARD_Y: i32 = 467;

pub fn screen_scan() -> Result<()> {

    let mut move_number = 0;
    let mut mover = "";
    let mut last_fen: String = "".to_string();
    let starting_position = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut castle_white_queenside = false;
    let mut castle_white_kingside = false;
    let mut castle_black_queenside = false;
    let mut castle_black_kingside = false;

    Command::new("screencapture")
        .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string()).expect("TODO: panic message");

    let best_match_top_left = find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/eight.png")?;
    let best_match_bottom_right = find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/h.png")?;

    let chessboard_x = best_match_top_left.x;
    let chessboard_y = best_match_top_left.y;

    let chessboard_width = best_match_bottom_right.x - chessboard_x + 67;

    loop {
        let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

        let roi = Rect::new(chessboard_x, chessboard_y, chessboard_width, chessboard_width);

        let mut chessboard_image = Mat::roi(&whole_screen, roi)?;
        let mut chessboard_image_resized = resize_square_image(&chessboard_image, Size::new(RESIZED_BOARD_IMAGE_WIDTH, RESIZED_BOARD_IMAGE_WIDTH))?;

        let squares = extract_chessboard_squares(&chessboard_image_resized)?;
        let mut piece_list = Vec::new();

        for (i, square) in squares.iter().enumerate() {
            let center = extract_center(&square)?;
            let piece = match_piece_image(&center)?;
            let square_num = i + 1;
            imgcodecs::imwrite(&*format!("/tmp/square-{}.png", square_num), &square, &Vector::new())?;

            if let Some(first_char) = piece.chars().next() {
                piece_list.push(first_char);
            } else {
                println!("The input string is empty.");
            }
        }

        if move_number == 0 && mover == "" {
            if piece_list[0] == 'r' {
                mover = "w"
            } else {
                mover = "b"
            }
        }

        let mut castle_string = "".to_string();
        if castle_white_queenside {
            castle_string.push('Q')
        }
        if castle_white_kingside {
            castle_string.push('K')
        }
        if castle_black_queenside {
            castle_string.push('q')
        }
        if castle_black_kingside {
            castle_string.push('k')
        }
        let fen = vec_to_fen(&piece_list, mover, &castle_string);
        if fen.split(" ").next() != last_fen.split(" ").next() {
            println!("{}", fen);

            let mut search_state = default_search_state();
            search_state.show_info = false;
            search_state.end_time = Instant::now().add(Duration::from_millis(1000));
            let position = get_position(&fen);
            if !test_bit(position.pieces[WHITE as usize].rook_bitboard, A1_BIT) {
                castle_white_queenside = true
            }
            if !test_bit(position.pieces[WHITE as usize].rook_bitboard, H1_BIT) {
                castle_white_kingside = true
            }
            if !test_bit(position.pieces[BLACK as usize].rook_bitboard, A8_BIT) {
                castle_black_queenside = true
            }
            if !test_bit(position.pieces[BLACK as usize].rook_bitboard, H8_BIT) {
                castle_black_kingside = true
            }
            let mv = iterative_deepening(&position, 100_u8, &mut search_state);
            println!("{}", algebraic_move_from_move(mv));

            move_number += 1;
            if mover == "w" {
                mover = "b"
            } else {
                mover = "w"
            }
            last_fen = fen.clone();
        }

        Command::new("screencapture")
            .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
            .output().map_err(|e| e.to_string()).expect("TODO: panic message");

    }

    Ok(())
}

fn vec_to_fen(pieces: &Vec<char>, mover: &str, castle_string: &str) -> String {
    let mut fen = String::new();
    let mut empty_squares = 0;

    for (index, &piece) in pieces.iter().enumerate() {
        if piece == ' ' {
            empty_squares += 1;
        } else {
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
                empty_squares = 0;
            }
            fen.push(piece);
        }

        if (index + 1) % 8 == 0 {
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
                empty_squares = 0;
            }
            if index < 56 {
                fen.push('/');
            }
        }
    }

    // Add the active player, castling rights, en passant square, halfmove clock, and fullmove number.
    // Update these according to the actual state of the game.
    fen.push_str(&*format!(" {} {} x 0 1", mover, castle_string));

    fen.replace("--------", "8")
        .replace("-------", "7")
        .replace("------", "6")
        .replace("-----", "5")
        .replace("----", "4")
        .replace("---", "3")
        .replace("--", "2")
        .replace("-", "1")
        .replace("x", "-")

}

fn extract_center(square: &Mat) -> Result<Mat, opencv::Error> {

    // Calculate the top-left point of the center region
    let top_left_x = 4;
    let top_left_y = 4;

    // Define the center region (ROI)
    let center_roi = Rect::new(top_left_x, top_left_y, 120, 120);

    // Extract the center region
    let center = Mat::roi(&square, center_roi)?;

    Ok(center)
}

fn resize_square_image(img: &Mat, size: Size_<i32>) -> Result<Mat, opencv::Error> {
    let new_size = size;
    let mut resized_img = Mat::default();
    resize(&img, &mut resized_img, new_size, 0.0, 0.0, INTER_LINEAR)?;
    Ok(resized_img)
}

fn in_range(i: i32, target: i32) -> RangeInclusive<i32> {
    let lower = target - 10;
    let upper = target + 10;
    lower..=upper
}


// rnbqkbnr/rnbqkbnr/pppppppp/pppppppp/PPPPPPPP/PPPPPPPP/RNBQKBNR/RNBQKBNR w - - 0 1

enum PixelColor {
    Black,
    White,
}

fn match_piece_image(image: &Mat) -> Result<String> {

    let white_pixels = count_pixels(image, PixelColor::White)?;
    let black_pixels = count_pixels(image, PixelColor::Black)?;

    let answer = if white_pixels + black_pixels == 0 {
        "-"
    } else if white_pixels == 0 && black_pixels > 0 {
       "p"
    } else if black_pixels == 0 && white_pixels > 0 {
        "P"
    } else if white_pixels < 250 {
        if black_pixels < 4500 {
            "r"
        } else {
            "n"
        }
    } else if white_pixels < 500 {
        if black_pixels < 3600 {
            "b"
        } else {
            "q"
        }
    } else if white_pixels > 800 && black_pixels > 2500 {
        "k"
    } else if white_pixels > 1500 && black_pixels < 750 {
        "P"
    } else if (2500..3000).contains(&white_pixels) && (500..1000).contains(&black_pixels) {
        "R"
    } else if (3400..3800).contains(&white_pixels) {
        if (900..1100).contains(&black_pixels) {
            "N"
        } else {
            "K"
        }
    } else if (1500..2000).contains(&white_pixels) && (1000..1200).contains(&black_pixels) {
        "B"
    } else if (2100..2300).contains(&white_pixels) && (1000..1200).contains(&black_pixels) {
        "Q"
    } else {
        "?"
    };

    Ok(answer.to_string())
}

fn extract_chessboard_squares(img: &Mat) -> Result<Vec<Mat>, opencv::Error> {
    let board_size = RESIZED_BOARD_IMAGE_WIDTH as usize;
    let square_size = board_size / 8;

    // Check if the input image has the expected size
    if img.rows() != board_size as i32 || img.cols() != board_size as i32 {
        println!("{} {}", img.rows(), img.cols());
        return Err(opencv::Error::new(0, "The input image must be a square with a size equal to 8 times the square size.".to_string()));
    }

    let mut squares = Vec::with_capacity(64);
    for row in 0..8 {
        for col in 0..8 {
            let x = col * square_size;
            let y = row * square_size;
            let square = Mat::roi(&img, Rect::new(x as i32, y as i32, square_size as i32, square_size as i32))?;
            squares.push(square);
        }
    }

    Ok(squares)
}

fn count_pixels(square: &Mat, color: PixelColor) -> Result<i32, opencv::Error> {

    let mut gray = Mat::default();
    cvt_color(&square, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut thresh = Mat::default();
    let threshold_value = match color {
        PixelColor::Black => 0.0,
        PixelColor::White => 250.0,
    };

    let threshold_type = match color {
        PixelColor::Black => imgproc::THRESH_BINARY_INV,
        PixelColor::White => imgproc::THRESH_BINARY,
    };

    threshold(
        &gray,
        &mut thresh,
        threshold_value,
        255.0,
        threshold_type,
    )?;

    let pixels = count_non_zero(&thresh)?;
    Ok(pixels)
}

fn extract_pieces(img: &Mat) -> Result<Vec<String>> {
    // Convert the image to grayscale
    let mut gray = Mat::default();
    cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Apply Gaussian blur to reduce noise
    let mut blurred = Mat::default();
    gaussian_blur(&gray, &mut blurred, Size::new(5, 5), 0.0, 0.0, BORDER_DEFAULT)?;

    // Apply adaptive thresholding to the image
    let mut thresh = Mat::default();
    imgproc::adaptive_threshold(
        &blurred,
        &mut thresh,
        255.0,
        imgproc::ADAPTIVE_THRESH_MEAN_C,
        imgproc::THRESH_BINARY,
        11,
        2.0,
    )?;

    // Find the contours of the thresholded image
    let mut contours: VectorOfVectorOfPoint = VectorOfVectorOfPoint::new();
    find_contours(
        &thresh,
        &mut contours,
        imgproc::RETR_LIST,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0.0 as i32, 0.0 as i32),
    )?;

    // Sort the contours by area and filter out small and large contours
    let mut areas = vec![];
    for contour in contours.iter() {
        let area = contour_area(&contour, false)?;
        areas.push((contour.clone(), area));
    }
    areas.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut pieces = Vec::new();
    let mut last_area = 0.0;
    for (contour, area) in areas.iter().rev() {
        // Filter out contours that are too small or too large
        if area < &(last_area / 2.0) || area > &(last_area * 2.0) {
            continue;
        }
        last_area = *area;

        // Find the bounding rectangle of the contour
        let rect = bounding_rect(contour)?;

        // Extract the sub-image within the bounding rectangle
        let subimg = Mat::roi(img, rect)?;
        let mut subimg_gray = Mat::default();
        imgproc::cvt_color(&subimg, &mut subimg_gray, imgproc::COLOR_BGR2GRAY, 0)?;

        // Threshold the sub-image and count the number of white pixels
        let mut subimg_thresh = Mat::default();
        imgproc::threshold(
            &subimg_gray,
            &mut subimg_thresh,
            0.0,
            255.0,
            imgproc::THRESH_BINARY_INV | imgproc::THRESH_OTSU,
        )?;
        let white_pixels = count_non_zero(&subimg_thresh)?;

        pieces.push(white_pixels.to_string());
    }

    Ok(pieces)
}

fn find_scaled_template(large_img_path: &str, template_img_path: &str) -> opencv::Result<Point> {
    // Load the images
    let large_img = imgcodecs::imread(large_img_path, imgcodecs::IMREAD_COLOR)?;
    let template_img = imgcodecs::imread(template_img_path, imgcodecs::IMREAD_COLOR)?;

    let mut best_match = Point::new(-1, -1);
    let mut best_match_val = f64::MAX;

    // Perform template matching
    let empty_mask = Mat::default();

    let mut result = Mat::default();
    imgproc::match_template(
        &large_img,
        &template_img,
        &mut result,
        imgproc::TM_SQDIFF_NORMED,
        &empty_mask,
    )?;


    let (mut min_val, mut max_val) = (0.,0.);
    let (mut min_loc, mut max_loc) = (Point::default(), Point::default());
    min_max_loc(
        &result,
        Option::from(&mut min_val),
        Option::from(&mut max_val),
        Option::from(&mut min_loc),
        Option::from(&mut max_loc),
        &empty_mask,
    )?;

    // Update the best match if necessary
    if min_val < best_match_val {
        best_match_val = min_val;
        best_match = min_loc;
    }

    Ok(best_match)
}
