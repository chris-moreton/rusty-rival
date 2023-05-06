use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc, ximgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::{Point, Point2f, Rect, Vector, ToInputArray, InputArray, count_non_zero, BORDER_DEFAULT, Size_, min_max_loc, subtract, no_array, pow, sum_elems, absdiff, norm, CV_8UC4};
use screenshots::Screen;
use std::{fs};
use std::error::Error;
use std::io::Read;
use std::ops::{Add, RangeInclusive};
use std::path::Path;
use std::process::{Command, exit};
use std::thread::sleep;
use opencv::types::VectorOfVectorOfPoint;
use crate::bitboards::{A1_BIT, A8_BIT, bit, H1_BIT, H8_BIT, test_bit};
use crate::fen::{algebraic_move_from_move, get_position};
use crate::moves::is_check;
use crate::search::iterative_deepening;
use crate::types::{BLACK, default_search_state, WHITE};
use crate::utils::invert_fen;
use scrap::{Capturer, Display};
use image::{ImageBuffer, Rgba};
use std::ffi::c_void;
use std::thread;
use std::fmt;

const RESIZED_BOARD_IMAGE_WIDTH: i32 = 1024;

pub fn screen_scan(flipped_board: bool) -> Result<()> {

    let mut move_number = 0;
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
        let start = Instant::now();
        let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;
        //println!("Reading screenshot file took: {:?}", start.elapsed());

        let roi = Rect::new(chessboard_x, chessboard_y, chessboard_width, chessboard_width);

        let start = Instant::now();
        let mut chessboard_image = Mat::roi(&whole_screen, roi)?;
        let mut chessboard_image_resized = resize_square_image(&chessboard_image, Size::new(RESIZED_BOARD_IMAGE_WIDTH, RESIZED_BOARD_IMAGE_WIDTH))?;
        //println!("Resizing chessboard_image took: {:?}", start.elapsed());

        let start = Instant::now();
        let squares = extract_chessboard_squares(&chessboard_image_resized)?;
        //println!("Extracting chessboard squares took: {:?}", start.elapsed());

        let mut piece_list = Vec::new();

        let start = Instant::now();
        if extract_pieces_from_resized_squares(squares, &mut piece_list) {
        //println!("Matching pieces took: {:?}", start.elapsed());

            if !piece_list.is_empty() {
                let start = Instant::now();

                let mut castle_string = "-".to_string();
                let mut fen = vec_to_fen(&piece_list, "w", &castle_string);

                if flipped_board {
                    piece_list.reverse();
                    fen = vec_to_fen(&piece_list, "w", &castle_string)
                }
                if fen.split(" ").next() != last_fen.split(" ").next() {
                    println!("{}", fen);
                    let mut search_state = default_search_state();
                    search_state.show_info = false;
                    search_state.end_time = Instant::now().add(Duration::from_millis(250));
                    let position = get_position(&fen);

                    if is_check(&position, BLACK) {
                        println!("Black is in check");
                    } else {
                        if !flipped_board {
                            let mv = iterative_deepening(&position, 100_u8, &mut search_state);
                            println!("White move is {}", algebraic_move_from_move(mv));
                        }
                    }
                    let fen = fen.replace(" w ", " b ");
                    let position = get_position(&fen);
                    if is_check(&position, WHITE) {
                        println!("White is in check");
                    } else {
                        if flipped_board {
                            let mv = iterative_deepening(&position, 100_u8, &mut search_state);
                            println!("Black move is {}", algebraic_move_from_move(mv));
                        }
                    }

                    move_number += 1;

                    last_fen = fen.clone();
                }
                //println!("Generating FEN took: {:?}", start.elapsed());
            }
        } else {
            println!("Failed to match pieces");
        }

        let start = Instant::now();
        Command::new("screencapture")
            .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
            .output().map_err(|e| e.to_string()).expect("TODO: panic message");
        //println!("Taking screenshot took: {:?}", start.elapsed());

    }
}

fn extract_pieces_from_resized_squares(squares: Vec<Mat>, piece_list: &mut Vec<char>) -> bool {
    for (i, square) in squares.iter().enumerate() {
        let center = extract_center(&square).unwrap();
        let piece = match_piece_image(&center);
        if piece != "" {
            if let Some(first_char) = piece.chars().next() {
                // if first_char == 'K' && piece_list.contains(&'K') {
                //     println!("White has more than one king");
                //     return false
                // }
                // if first_char == 'k' && piece_list.contains(&'k') {
                //     println!("Black has more than one king");
                //     return false
                // }

                piece_list.push(first_char);
            } else {
                println!("The input string is empty.");
            }
        } else {
            return false
        }
    }
    return true
}

fn is_screenshot_ready(chessboard_x: i32, chessboard_y: i32, chessboard_width: i32) -> Result<bool> {

    Command::new("screencapture")
        .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string()).expect("TODO: panic message");

    sleep(Duration::from_millis(50));

    Command::new("screencapture")
        .args(&["-D1", "-x", "-t", "png", "/tmp/screenshot2.png"])
        .output().map_err(|e| e.to_string()).expect("TODO: panic message");

    let s = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;
    let s2 = imgcodecs::imread("/tmp/screenshot2.png", imgcodecs::IMREAD_COLOR)?;

    // let s = capture()?;
    // sleep(Duration::from_millis(50));
    // let s2 = capture()?;

    let roi = Rect::new(chessboard_x, chessboard_y, chessboard_width, chessboard_width);

    let mut c = Mat::roi(&s, roi)?;
    let mut c2 = Mat::roi(&s2, roi)?;

    imgcodecs::imwrite("/tmp/board.png",&c, &Vector::new())?;
    imgcodecs::imwrite("/tmp/board2.png", &c2, &Vector::new())?;

    return Ok(compare_files("/tmp/board.png", "/tmp/board2.png"))

}

fn capture() -> Result<Mat> {
    let display = Display::primary();

    match display {
        Ok(display) => {
            let mut capturer = Capturer::new(display);

            match capturer {
                Ok(mut capturer) => {
                    let (width, height) = (capturer.width(), capturer.height());
                    let frame = loop {
                        match capturer.frame() {
                            Ok(frame) => break frame,
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            },
                            Err(e) => {
                                println!("Failed to capture frame: {}", e);
                                exit(1)
                            },
                        }
                    };

                    let buffer =
                        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, frame.to_vec())
                            .ok_or("Failed to create image buffer");

                    match buffer {
                        Ok(buffer) => {
                            let mat = imgbuf_to_mat(&buffer);
                            match mat {
                                Ok(mat) => Ok(mat),
                                Err(e) => {
                                    println!("Failed to create mat: {}", e);
                                    exit(1)
                                }
                            }
                        },
                        Err(e) => {
                            println!("Failed to create image buffer: {}", e);
                            exit(1)
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to create capturer: {}", e);
                    exit(1)
                }
            }
        },
        Err(e) => {
            println!("Failed to get display");
            exit(1)
        }
    }

}

fn imgbuf_to_mat(imgbuf: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Mat, Box<dyn Error>> {
    let (width, height) = (imgbuf.width(), imgbuf.height());
    let data = imgbuf.as_flat_samples();
    let step = 4 * width;

    let mat: Mat = unsafe {
        Mat::new_rows_cols_with_data(height as i32, width as i32, CV_8UC4, data.as_slice().as_ptr() as *mut c_void, step as usize)?
    };

    Ok(mat)
}

fn compare_files(path1: &str, path2: &str) -> bool {
    let mut file1 = fs::File::open(path1).expect("");
    let mut file2 = fs::File::open(path2).expect("");

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    file1.read_to_end(&mut buf1).expect("TODO: panic message");
    file2.read_to_end(&mut buf2).expect("TODO: panic message");

    buf1 == buf2
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

fn in_range(i: i32, target: i32) -> RangeInclusive<i32> {
    let lower = target - 10;
    let upper = target + 10;
    lower..=upper
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
