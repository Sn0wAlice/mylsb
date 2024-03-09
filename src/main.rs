extern crate image;

use image::{DynamicImage, GenericImageView};
use std::fs::File;
use std::io::{Read, Write};

/// Encodes a message into the LSB of the pixel values of an image
fn encode_lsb(image_path: &str, message: Vec<u8>) -> Result<(), &'static str> {
    // Load the image
    let img = image::open(image_path).map_err(|_| "Failed to open image")?;

    // Convert the message to binary
    let binary_message = message.iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1 == 1))
        .map(|bit| bit as u8)
        .collect::<Vec<u8>>();

    // Ensure the message can fit in the image
    if binary_message.len() > img.to_rgb8().into_raw().len() * 8 {
        return Err("Message is too long to encode in the image");
    }

    // Encode the message into the image
    let mut index = 0;
    let mut new_pixels = img.to_rgb8().into_raw();
    for bit in binary_message {
        if index >= new_pixels.len() {
            break;
        }
        let pixel = &mut new_pixels[index];
        *pixel = (*pixel & 0xFE) | bit;
        index += 1;
    }

    // Create a new image with the modified pixel data
    let (width, height) = img.dimensions();
    let encoded_img = DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, new_pixels).unwrap());

    // Save the encoded image
    let output_path = format!("encoded_{}", image_path);
    encoded_img.save(output_path.clone()).map_err(|_| "Failed to save encoded image")?;

    println!("Encoded image saved to {}", output_path);

    Ok(())
}

/// Decodes a message from the LSB of the pixel values of an image
fn decode_lsb(image_path: &str) -> Result<Vec<u8>, &'static str> {
    // Load the image
    let img = image::open(image_path).map_err(|_| "Failed to open image")?;

    // Extract LSBs from pixel values
    let binary_message = img.to_rgb8().into_raw().iter()
        .map(|byte| byte & 1)
        .collect::<Vec<u8>>();

    // Convert binary message to string
    let message = binary_message.chunks(8)
        .map(|byte| byte.iter().fold(0, |acc, &bit| (acc << 1) | bit as u8))
        .collect::<Vec<u8>>();


    Ok(message)
}

fn main() {


    // check if first argument is encode or decode or else
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run [encode|decode|help]"); 
        return;
    }

    let command = &args[1];

    if command == "encode" {
        let image_path = &args[2];
        let secret_file_path = &args[3];
        let mut file = File::open(secret_file_path).expect("Failed to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read file");
        encode_lsb(image_path, buffer).expect("Failed to encode message into image");
        return;
    } else if command == "decode" {
        let image_path = &args[2];
        let output_file_path = &args[3];
        let decoded_message = decode_lsb(image_path).expect("Failed to decode message from image");
        let mut file = File::create(output_file_path).expect("Failed to create file");
        file.write_all(&decoded_message).expect("Failed to write to file");
        return;
    } else if command == "help" {
        println!("Usage: mylsb [encode|decode|help]"); 
        print!("encode: ./mylsb <image_path> <secret_file_path>\n");
        print!("decode: ./mylsb <image_path> <output_file_path>\n");
        return;
    } else {
        println!("Usage: ./mylsb [encode|decode|help]"); 
        return;
    }
}
