use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::{Point, Point2f, Rect, Vector, ToInputArray, InputArray, count_non_zero, BORDER_DEFAULT, Size_};
use screenshots::Screen;
use std::{fs};
use std::ops::RangeInclusive;
use std::process::Command;
use opencv::types::VectorOfVectorOfPoint;

const BOARD_IMAGE_WIDTH: i32 = 2932;
const RESIZED_BOARD_IMAGE_WIDTH: i32 = 1024;
const CHESSBOARD_X: i32 = 1322;
const CHESSBOARD_Y: i32 = 467;

pub fn screen_scan() -> Result<()> {

    let start = Instant::now();

    let output = Command::new("screencapture")
        .args(&["-D1", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string());

    println!("Running time: {:?}", start.elapsed());

    // Define the size of the chessboard
    let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

    let roi = Rect::new(CHESSBOARD_X, CHESSBOARD_Y, BOARD_IMAGE_WIDTH, BOARD_IMAGE_WIDTH);

    let mut chessboard_image = Mat::roi(&whole_screen, roi)?;
    let mut chessboard_image_resized = resize_square_image(&chessboard_image, Size::new(RESIZED_BOARD_IMAGE_WIDTH, RESIZED_BOARD_IMAGE_WIDTH))?;

    let squares = extract_chessboard_squares(&chessboard_image_resized)?;

    for (i, square) in squares.iter().enumerate() {
        let center = extract_center(&square)?;
        let black_pixels = count_black_pixels(&center)?;
        let piece = get_piece_string_from_pixels(black_pixels)?;
        let center_of_square = resize_square_image(&square, Size::new(64, 64))?;
        let square_num = i + 1;
        imgcodecs::imwrite(&*format!("/tmp/square-{}.png", square_num), &square, &Vector::new())?;
        println!("Square {}: {} black pixels - my guess is {}", square_num, black_pixels, piece);
    }

    Ok(())
}

fn extract_center(square: &Mat) -> Result<Mat, opencv::Error> {
    let square_size = 128;
    let center_size = 96;

    // Calculate the top-left point of the center region
    let top_left_x = (square_size - center_size) / 2;
    let top_left_y = (square_size - center_size) / 2;

    // Define the center region (ROI)
    let center_roi = Rect::new(top_left_x, top_left_y, center_size, center_size);

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

fn get_piece_string_from_pixels(white_pixels: i32) -> Result<String> {
    let piece_ranges = [
        (4255, "Black rook on light square"),
        (4807, "Black knight on dark square"),
        (3477, "Black bishop on light square"),
        (3899, "Black queen on dark square"),
        (3284, "Black king on light square"),
        (3554, "Black bishop on dark square"),
        (4358, "Black rook on dark square"),
        (4710, "Black knight on light square"),
        (2936, "Black pawn on light square"),
        (2990, "Black pawn on dark square"),
        (1176, "White rook on light square"),
        (1301, "White knight on dark square"),
        (1383, "White bishop on light square"),
        (1526, "White queen on dark square"),
        (1432, "White king on light square"),
        (1459, "White bishop on dark square"),
        (1229, "White knight on light square"),
        (790, "White pawn on light square"),
        (830, "White pawn on dark square"),
    ];

    let piece = if white_pixels == 0 {
        "empty"
    } else {
        let mut found_piece = "unknown";
        for (target, piece_name) in piece_ranges.iter() {
            if in_range(white_pixels, *target).contains(&white_pixels) {
                found_piece = piece_name;
                break;
            }
        }
        found_piece
    };

    Ok(piece.to_string())
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

fn count_black_pixels(square: &Mat) -> Result<i32, opencv::Error> {
    let mut gray = Mat::default();
    cvt_color(&square, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut thresh = Mat::default();
    let threshold_value = 10.0; // Set the threshold value
    threshold(
        &gray,
        &mut thresh,
        threshold_value,
        255.0,
        imgproc::THRESH_BINARY_INV,
    )?;

    let black_pixels = count_non_zero(&thresh)?;
    Ok(black_pixels)
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
