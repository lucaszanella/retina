use ffmpeg::decoder::{FfmpegSoftwareDecoder, Codec};
use anyhow::{anyhow, Error};
use futures::StreamExt;
use retina::codec::CodecItem;
use log::{error, info};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Source {
    #[structopt(long, parse(try_from_str))]
    url: url::Url,

    #[structopt(long, requires = "password")]
    username: Option<String>,

    #[structopt(long, requires = "username")]
    password: Option<String>,
}

fn init_logging() -> mylog::Handle {
    let h = mylog::Builder::new()
        .set_format(
            ::std::env::var("MOONFIRE_FORMAT")
                .map_err(|_| ())
                .and_then(|s| mylog::Format::from_str(&s))
                .unwrap_or(mylog::Format::Google),
        )
        .set_spec(::std::env::var("MOONFIRE_LOG").as_deref().unwrap_or("info"))
        .build();
    h.clone().install().unwrap();
    h
}

#[tokio::main]
async fn main() {
    //let mut h = init_logging();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if let Err(e) = {
        //let _a = h.async_scope();
        let opts = Source::from_args();
        run(opts).await
    } {
        error!("Fatal: {}", e);
        std::process::exit(1);
    }
    info!("Done");
}

/// Interpets the `username` and `password` of a [Source].
fn creds(
    username: Option<String>,
    password: Option<String>,
) -> Option<retina::client::Credentials> {
    match (username, password) {
        (Some(username), Some(password)) => {
            Some(retina::client::Credentials { username, password })
        }
        (None, None) => None,
        _ => unreachable!(), // structopt/clap enforce username and password's mutual "requires".
    }
}

async fn run(opts: Source) -> Result<(), Error> {
    let ffmpeg_decoder = FfmpegSoftwareDecoder::new_blocking(Codec::H264).unwrap();
    let stop = tokio::signal::ctrl_c();

    let creds = creds(opts.username, opts.password);
    let mut session = retina::client::Session::describe(
        opts.url,
        retina::client::SessionOptions::default()
            .creds(creds)
            .user_agent("Retina metadata example".to_owned()),
    )
    .await?;
    let onvif_stream_i = session
        .streams()
        .iter()
        .position(|s| {
            //info!("stream: {:?}", s);
            s.media=="video"
        })
        .ok_or_else(|| anyhow!("couldn't find onvif stream"))?;
    session.setup(onvif_stream_i).await?;
    let session = session
        .play(retina::client::PlayOptions::default().ignore_zero_seq(true))
        .await?
        .demuxed()?;

    tokio::pin!(session);
    tokio::pin!(stop);
    loop {
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {
                        info!("{}: {}\n", &m.timestamp, std::str::from_utf8(&m.data[..]).unwrap());
                    },
                    CodecItem::VideoFrame(f) => {
                        info!("<<< {} bytes", f.data().len());
                        ffmpeg_decoder.send(&f.data()).unwrap();
                        ffmpeg_decoder.receive(&mut |maybe_packet|{
                            if let Some(packet) = maybe_packet {
                                info!("decoded frame, format: {:?}", packet.format());
                            } else {
                                info!("receive: no packet");
                            }
                        }).unwrap();
                    }
                    _ => continue,
                };
            },
            _ = &mut stop => {
                break;
            },
        }
    }
    Ok(())
}
