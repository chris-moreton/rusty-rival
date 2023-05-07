use opencv::prelude::*;

use opencv::{core::{Mat, Size}, imgcodecs, imgproc::*, Result, imgproc};
use std::time::{Duration, Instant};
use opencv::core::{Point, Rect, count_non_zero, Size_, min_max_loc};
use std::ops::{Add};
use std::process::{Command};
use crate::fen::{algebraic_move_from_move, get_position};
use crate::moves::is_check;
use crate::search::iterative_deepening;
use crate::types::{BLACK, default_search_state, WHITE};
use colored::*;
use figlet_rs::FIGfont;
use crossterm::{
    terminal::{self, ClearType},
    execute,
};
use std::io::{stdout};

const RESIZED_BOARD_IMAGE_WIDTH: i32 = 1024;

pub fn screen_scan(flipped_board: bool) -> Result<()> {

    let mut ticker = 0;
    let mut last_fen: String = "".to_string();

    if flipped_board {
        println!("Board is flipped");
    } else {
        println!("Board is not flipped");
    }
    Command::new("screencapture")
        .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string()).expect("TODO: panic message");

    let best_match_top_left = if flipped_board {
        find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/one.png")?
    } else {
        find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/eight.png")?
    };

    let chessboard_x = best_match_top_left.x;
    let chessboard_y = best_match_top_left.y;

    let best_match_bottom_right = if flipped_board {
        find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/a.png")?
    } else {
        find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/h.png")?
    };

    let chessboard_width = best_match_bottom_right.x - chessboard_x + 67;

    println!("Chessboard x: {}, y: {}, width: {}", chessboard_x, chessboard_y, chessboard_width);

    loop {

        let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

        let roi = Rect::new(chessboard_x, chessboard_y, chessboard_width, chessboard_width);

        let chessboard_image = Mat::roi(&whole_screen, roi)?;
        let chessboard_image_resized = resize_square_image(&chessboard_image, Size::new(RESIZED_BOARD_IMAGE_WIDTH, RESIZED_BOARD_IMAGE_WIDTH))?;

        let squares = extract_chessboard_squares(&chessboard_image_resized)?;

        let mut piece_list = Vec::new();

        if extract_pieces_from_resized_squares(squares, &mut piece_list) {

            if !piece_list.is_empty() {

                let castle_string = "-".to_string();
                let mut fen = vec_to_fen(&piece_list, "w", &castle_string);

                if flipped_board {
                    piece_list.reverse();
                    fen = vec_to_fen(&piece_list, "w", &castle_string)
                }
                if fen.split(" ").next() != last_fen.split(" ").next() {
                    // println!("{}", fen);
                    let mut search_state = default_search_state();
                    search_state.show_info = false;
                    search_state.end_time = Instant::now().add(Duration::from_millis(250));
                    let position = get_position(&fen);

                    if !is_check(&position, BLACK) {
                        if !flipped_board {
                            let mv = iterative_deepening(&position, 100_u8, &mut search_state);
                            ticker += 1;
                            show_move_text(algebraic_move_from_move(mv), ticker);
                        }
                    }
                    let fen = fen.replace(" w ", " b ");
                    let position = get_position(&fen);
                    if !is_check(&position, WHITE) {
                        if flipped_board {
                            let mv = iterative_deepening(&position, 100_u8, &mut search_state);
                            ticker += 1;
                            show_move_text(algebraic_move_from_move(mv), ticker);
                        }
                    }

                    last_fen = fen.clone();
                }
                //println!("Generating FEN took: {:?}", start.elapsed());
            }
        }

        Command::new("screencapture")
            .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
            .output().map_err(|e| e.to_string()).expect("TODO: panic message");
    }
}

fn extract_pieces_from_resized_squares(squares: Vec<Mat>, piece_list: &mut Vec<char>) -> bool {
    for (_, square) in squares.iter().enumerate() {
        let center = extract_center(&square).unwrap();
        let piece = match_piece_image(&center);
        if piece != "" {
            if let Some(first_char) = piece.chars().next() {
                if first_char == 'K' && piece_list.contains(&'K') {
                    return false
                }
                if first_char == 'k' && piece_list.contains(&'k') {
                    return false
                }

                piece_list.push(first_char);
            } else {
                println!("The input string is empty.");
            }
        } else {
            return false
        }
    }
    return piece_list.contains(&'k') && piece_list.contains(&'K');
}

fn show_move_text(text: String, number: u32) {

    let color_index = number % 4;

    execute!(stdout(), terminal::Clear(ClearType::All)).unwrap();

    // Load a FIGfont
    let font = FIGfont::standard().unwrap();

    // Generate the banner-style text
    let banner = font.convert(text.as_str()).unwrap().to_string();

    // Print the banner-style text
    let coloured_text = match color_index {
        0 => banner.red(),
        1 => banner.green(),
        2 => banner.blue(),
        3 => banner.yellow(),
        _ => banner.yellow(), // This case should never be reached
    };

    println!("{}", coloured_text);
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

fn extract_top_quarter(square: &Mat) -> Result<Mat, opencv::Error> {

    // Calculate the top-left point of the center region
    let top_left_x = 0;
    let top_left_y = 0;

    // Define the center region (ROI)
    let center_roi = Rect::new(top_left_x, top_left_y, 120, 30);

    // Extract the center region
    let center = Mat::roi(&square, center_roi)?;

    Ok(center)
}

fn resize_square_image(img: &Mat, size: Size_<i32>) -> Result<Mat, opencv::Error> {
    let new_size = size;
    let mut resized_img = Mat::default();
    resize(&img, &mut resized_img, new_size, 0.0, 0.0, INTER_AREA)?;
    Ok(resized_img)
}

// rnbqkbnr/rnbqkbnr/pppppppp/pppppppp/PPPPPPPP/PPPPPPPP/RNBQKBNR/RNBQKBNR w - - 0 1
// rnbqkbnr/rnbqkbnr/pppppppp/pppppppp/PPPPPPPP/PPPPPPPP/RNBQKBNR/RNBQKBNR w 1 - 0 1

enum PixelColor {
    Black,
    White,
}

fn match_piece_image(image: &Mat) -> String {

    let white_pixels = count_pixels(image, PixelColor::White).unwrap();
    let black_pixels = count_pixels(image, PixelColor::Black).unwrap();

    let answer = if (white_pixels + black_pixels) == 0 {
        "-".to_string()
    } else {
        if white_pixels < 1200 {
            match_black_piece(white_pixels, black_pixels)
        } else {
            match_white_piece(white_pixels, black_pixels, image)
        }
    };

    //println!("{},{},{}", answer.to_string(), white_pixels, black_pixels);
    answer
}

fn match_white_piece(white_pixels: i32, black_pixels: i32, image: &Mat) -> String {
    match (white_pixels, black_pixels) {
        (..=1900, ..=750) => "P",
        (..=1900, 751..) => "B",
        (..=2500, _) => "Q",
        (..=3000, _) => "R",
        (3000.., 600..) => {
            if is_knight(image) {
                "N"
            } else {
                "K"
            }
        },
        _ => {
            "X"
        }
    }.to_string()
}

fn match_black_piece(white_pixels: i32, black_pixels: i32) -> String {
    match (white_pixels, black_pixels) {
        (0, _) => "p",
        (_, 4300..) => "n",
        (_, 4000..) => "r",
        (_, 3450..) => "q",
        (_, 3000..) => "b",
        (_, 2500..) => "k",
        _ => {
            "X"
        }
    }.to_string()
}

fn is_knight(square: &Mat) -> bool {
    let image = extract_top_quarter(&square).unwrap();

    let white_pixels = count_pixels(&image, PixelColor::White).unwrap();

    return white_pixels > 0
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

fn find_scaled_template(large_img_path: &str, template_img_path: &str) -> opencv::Result<Point> {
    // Load the images
    let large_img = imgcodecs::imread(large_img_path, imgcodecs::IMREAD_COLOR)?;
    let template_img = imgcodecs::imread(template_img_path, imgcodecs::IMREAD_COLOR)?;

    let mut best_match = Point::new(-1, -1);
    let best_match_val = f64::MAX;

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
        best_match = min_loc;
    }

    Ok(best_match)
}
