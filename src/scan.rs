use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::{Point, Point2f, Rect, Vector, ToInputArray, InputArray, count_non_zero};
use screenshots::Screen;
use std::{fs};
use std::process::Command;
use opencv::types::VectorOfVectorOfPoint;

pub fn screen_scan() -> Result<()> {

    let start = Instant::now();

    let output = Command::new("screencapture")
        .args(&["-D1", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string());

    // 1322 467 2932
    println!("Running time: {:?}", start.elapsed());

    // Define the size of the chessboard
    let size = Size::new(8, 8);

    let whole_screen = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

    let roi = Rect::new(1322, 467, 2932, 2932);

    let mut frame = Mat::roi(&whole_screen, roi)?;

    // Show the frame in the "Screenshot" window
    highgui::imshow("Screenshot", &frame)?;

    // Convert the frame to grayscale
    let mut gray = Mat::default();
    cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Find the chessboard corners
    let corners = vec![
        Point::new(0.0 as i32, 0.0 as i32),
        Point::new(2932.0 as i32, 0.0 as i32),
        Point::new(2932.0 as i32, 2932.0 as i32),
        Point::new(0.0 as i32, 2932.0 as i32),
    ];

    let mut points = Vec::new();
    for corner in corners.iter() {
        points.push(Point::new(corner.x as i32, corner.y as i32));
    }
    let points = points.iter().map(|&p| Point::from(p)).collect::<Vec<_>>();

    let mut points_sorted = points.clone();
    points_sorted.sort_by_key(|p| (p.y, p.x));

    let mut pieces = String::new();
    for i in 0..size.area() {
        let piece = match (i / size.width, i % size.width) {
            (0, 0) | (0, 7) => 'R',
            (0, 1) | (0, 6) => 'N',
            (0, 2) | (0, 5) => 'B',
            (0, 3) => 'Q',
            (0, 4) => 'K',
            (1, _) => 'P',
            (6, _) => 'p',
            (7, 0) | (7, 7) => 'r',
            (7, 1) | (7, 6) => 'n',
            (7, 2) | (7, 5) => 'b',
            (7, 3) => 'q',
            (7, 4) => 'k',
            _ => ' ',
        };
        pieces.push(piece);
    }

    // Generate the FEN string
    let fen = format!(
        "{} {} {} {} {} {}",
        pieces,
        "-",
        "KQkq",
        "-",
        "0",
        "1"
    );
    println!("{}", fen);

    imgcodecs::imwrite("/tmp/screenshot-gray.png", &gray, &Vector::new())?;

    let pieces = extract_pieces(&frame)?;

    println!("{:?}", pieces);

    Ok(())
}

fn extract_pieces(img: &Mat) -> Result<Vec<char>> {
    // Convert the image to grayscale
    let mut gray = Mat::default();
    cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Apply adaptive thresholding to the image
    let mut thresh = Mat::default();
    imgproc::adaptive_threshold(
        &gray,
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

        // Classify the piece based on the number of white pixels
        let piece = match white_pixels {
            0..=300 => 'P',
            301..=600 => 'N',
            601..=900 => 'B',
            901..=1200 => 'R',
            1201..=1500 => 'Q',
            _ => 'K',
        };
        pieces.push(piece);
    }

    Ok(pieces)
}
