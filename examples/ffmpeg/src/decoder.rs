use ffmpeg_next::codec::Context as CodecContext;
use ffmpeg_next::decoder::Opened;
use ffmpeg_next::Error as FfmpegError;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::cell::RefCell;

pub struct FfmpegSoftwareDecoder {
    pub codec: Option<Codec>,
    pub decoder: Option<RefCell<Opened>>,
}

unsafe impl Send for FfmpegSoftwareDecoder {}

#[derive(Debug, Clone)]
pub enum Codec {
    H264,
    H265,
    MPEG2,
    MPEG4,
    VP9,
}

#[derive(Debug)]
pub enum FfmpegCreationError {
    CodecNotFound(String),
    CodecNotSupported(Codec),
    DecoderOpenError(FfmpegError),
    DecoderOpenErrorStr(String),
    Unknown(String),
}

#[derive(Debug)]
pub enum FfmpegSoftwareDecoderSendError {
    MissingOnConsume,
    MissingDecoder,
    /// Rtsp client should always produce a packet, but `OnConsume` is flexible
    /// and works with `Option<EncodedPacket>`
    NoPacket,
}

#[derive(Debug)]
pub enum FfmpegSoftwareDecoderReceiveError {
    MissingOnProduce,
}

impl FfmpegSoftwareDecoder {
    pub fn new_blocking(codec: Codec) -> Result<FfmpegSoftwareDecoder, FfmpegCreationError> {
        let mut codec_context = Some(CodecContext::new());
        let video_decoder = FfmpegSoftwareDecoder::open_decoder(
            &codec,
            codec_context
                .take()
                .ok_or(FfmpegCreationError::DecoderOpenErrorStr(
                    "no codec_context".to_string(),
                ))?,
        )?;
        Ok(FfmpegSoftwareDecoder {
            codec: Some(codec),
            decoder: Some(video_decoder),
        })
    }

    pub fn open_decoder(
        codec: &Codec,
        context: CodecContext,
    ) -> Result<RefCell<Opened>, FfmpegCreationError> {
        let codec_string = match codec {
            Codec::H264 => "h264",
            _ => return Err(FfmpegCreationError::CodecNotSupported(codec.clone())),
        };
        let codec = ffmpeg_next::codec::decoder::find_by_name(codec_string);
        let codec = codec.ok_or(FfmpegCreationError::CodecNotFound(
            format!("codec not found: {}", codec_string).into(),
        ))?;
        let codec_id = codec.get_id();
        let mut video_decoder = context
            .decoder()
            .open_as(codec)
            .map_err(|e| FfmpegCreationError::DecoderOpenError(e))?;
        video_decoder.init_parser(codec_id);
        Ok(RefCell::new(video_decoder))
    }

    pub fn send<'a, 'b>(&self, encoded_packet: &[u8]) -> Result<(), String> {
        let mut encoded_packet_size = encoded_packet.len();
        let mut encoded_packet_index = 0;
        let decoder = self.decoder.as_ref().ok_or("missing decoder".to_string())?;
        while encoded_packet_size > 0 {
            match decoder.borrow_mut().parse2(
                &encoded_packet[encoded_packet_index..],
                None,
                None,
                0,
            ) {
                Ok((bytes_parsed, parsed_packet)) => {
                    encoded_packet_index += bytes_parsed as usize;
                    encoded_packet_size -= bytes_parsed as usize;
                    match parsed_packet {
                        Some(parsed_packet) => {
                            let packet = ffmpeg_next::packet::Packet::copy(&parsed_packet);
                            if let Err(e) = decoder.borrow_mut().send_packet(&packet) {
                                //warn
                                error!("send_packet error: {:?}", e);
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("parse2 error: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn receive(
        &self,
        on_packet: &mut dyn FnMut(Option<ffmpeg_next::util::frame::Video>),
    ) -> Result<(), String> {
        let decoder = self.decoder.as_ref().ok_or("missing decoder".to_string())?;
        let mut decoded_packet = ffmpeg_next::util::frame::Video::empty();
        match decoder.borrow_mut().receive_frame(&mut decoded_packet) {
            Ok(_) => {
                let w = decoded_packet.width();
                let h = decoded_packet.height();
                on_packet(Some(decoded_packet));
                if w > 0 && h > 0 {
                    //println!("decoded!");
                    info!("decoded");
                } else {
                    error!("width and height of frame are wrong. w:{}, h:{}", w, h)
                }
            }
            Err(ffmpeg_next::Error::Other {
                errno: libc::EAGAIN,
            }) => {}
            Err(e) => error!("unknown decoding error: {}", e),
        }
        Ok(())
    }
}
