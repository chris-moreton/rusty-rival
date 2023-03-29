use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc, ximgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::{Point, Point2f, Rect, Vector, ToInputArray, InputArray, count_non_zero, BORDER_DEFAULT, Size_, min_max_loc, subtract, no_array, pow, sum_elems};
use screenshots::Screen;
use std::{fs};
use std::ops::RangeInclusive;
use std::process::Command;
use std::thread::sleep;
use opencv::types::VectorOfVectorOfPoint;

const BOARD_IMAGE_WIDTH: i32 = 2932;
const RESIZED_BOARD_IMAGE_WIDTH: i32 = 1024;
const CHESSBOARD_X: i32 = 1322;
const CHESSBOARD_Y: i32 = 467;

pub fn screen_scan() -> Result<()> {

    let mut move_number = 0;
    let mut mover = "";
    let mut fen_list = Vec::new();

    fen_list.push("Start".parse().unwrap());

    loop {
        Command::new("screencapture")
            .args(&["-D1", "-t", "png", "/tmp/screenshot.png"])
            .output().map_err(|e| e.to_string()).expect("TODO: panic message");

        let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

        let best_match_top_left = find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/eight.png")?;
        let best_match_bottom_right = find_scaled_template("/tmp/screenshot.png", "/Users/chris/git/chris-moreton/resources/h.png")?;

        let chessboard_x = best_match_top_left.x;
        let chessboard_y = best_match_top_left.y;

        let chessboard_width = best_match_bottom_right.x - chessboard_x + 67;

        let roi = Rect::new(chessboard_x, chessboard_y, chessboard_width, chessboard_width);

        let mut chessboard_image = Mat::roi(&whole_screen, roi)?;
        let mut chessboard_image_resized = resize_square_image(&chessboard_image, Size::new(RESIZED_BOARD_IMAGE_WIDTH, RESIZED_BOARD_IMAGE_WIDTH))?;

        imgcodecs::imwrite(&*format!("/tmp/chessboard-cropped.png"), &chessboard_image, &Vector::new())?;
        imgcodecs::imwrite(&*format!("/tmp/chessboard-resized.png"), &chessboard_image_resized, &Vector::new())?;

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

        mover = if move_number == 0 {
            if piece_list[0] == 'r' {
                "w"
            } else {
                "b"
            }
        } else {
            if mover == "w" {
                "b"
            } else {
                "w"
            }
        };

        let fen = vec_to_fen(&piece_list, mover);
        if let Some(last_fen) = fen_list.last() {
            if fen != *last_fen {
                println!("{}", fen);
                move_number += 1;
                fen_list.push(fen.to_string());
            }
        }

        sleep(Duration::from_millis(2000))
    }

    Ok(())
}

fn vec_to_fen(pieces: &Vec<char>, mover: &str) -> String {
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
    fen.push_str(&*format!(" {} KQkq - 0 1", mover));

    fen
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
