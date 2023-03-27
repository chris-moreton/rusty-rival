use std::arch::aarch64::int32x2_t;
use opencv::prelude::*;

use opencv::{core::{Mat, Size}, highgui, imgcodecs, imgproc::*, videoio, Result, imgproc};
use std::time::{Duration, Instant};
use opencv::calib3d::{CALIB_CB_ADAPTIVE_THRESH, CALIB_CB_NORMALIZE_IMAGE, draw_chessboard_corners, find_chessboard_corners};
use opencv::core::Vector;
use screenshots::Screen;
use std::{fs};
use std::process::Command;

pub fn screen_scan() -> Result<()> {

    let start = Instant::now();

    let output = Command::new("screencapture")
        .args(&["-D1", "-t", "png", "/tmp/screenshot.png"])
        .output().map_err(|e| e.to_string());

    println!("Running time: {:?}", start.elapsed());

    // Define the size of the chessboard
    let size = Size::new(8, 8);

    let mut frame = imgcodecs::imread("/tmp/screenshot.png", imgcodecs::IMREAD_COLOR)?;

    // Show the frame in the "Screenshot" window
    highgui::imshow("Screenshot", &frame)?;

    // Convert the frame to grayscale
    let mut gray = Mat::default();
    cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Find the chessboard corners
    let mut corners = Mat::default();
    let found = find_chessboard_corners(
        &gray,
        size,
        &mut corners,0,
    )?;

    imgcodecs::imwrite("/tmp/screenshot-gray.png", &gray, &Vector::new())?;

    // If the corners are found, draw them on the original frame
    draw_chessboard_corners(&mut frame, size, &corners, found)?;

    // Save the image to a file
    imgcodecs::imwrite("/tmp/chessboard-2.png", &frame, &Vector::new())?;

    Ok(())
}
