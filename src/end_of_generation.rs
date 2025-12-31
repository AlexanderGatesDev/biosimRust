// end_of_generation.rs
// At the end of each generation, we save a video file and print genomic statistics

use crate::params::Params;
use crate::image_writer::ImageWriter;
use std::process::Command;

pub fn end_of_generation(
    generation: u32,
    params: &Params,
    image_writer: &mut ImageWriter,
) {
    if params.save_video
        && ((generation % params.video_stride == 0)
            || generation <= params.video_save_first_frames
            || (generation >= params.parameter_change_generation_number
                && generation
                    <= params.parameter_change_generation_number
                        + params.video_save_first_frames))
    {
        image_writer.save_generation_video(generation, params);
    }

    if params.update_graph_log
        && (generation == 1 || (generation % params.update_graph_log_stride == 0))
    {
        // Execute the graph log update command
        let parts: Vec<&str> = params.graph_log_update_command.split_whitespace().collect();
        if !parts.is_empty() {
            let mut cmd = Command::new(parts[0]);
            for arg in parts.iter().skip(1) {
                cmd.arg(arg);
            }
            let _ = cmd.output();
        }
    }
}

