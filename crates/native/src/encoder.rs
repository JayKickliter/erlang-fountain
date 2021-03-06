use crate::atoms::{seeded, edges, random, systematic};
use rustler::{Atom, OwnedBinary, Term, Encoder, Env, Error, Binary, NifResult};
use rustler::resource::ResourceArc;
use fountaincode::encoder::Encoder as FountainEncoder;
use fountaincode::types::{EncoderType, DropType};
use std::sync::RwLock;
use std::io::Write;

pub struct EncoderRes {
    pub encoder: RwLock<FountainEncoder>
}

pub fn load(env: Env) -> bool {
    rustler::resource!(EncoderRes, env);
    true
}

#[rustler::nif(name = "new_encoder")]
fn new_encoder(
    data: Binary,
    blocksize: usize,
    encodertype: Atom) -> NifResult<ResourceArc<EncoderRes>> {
    if encodertype == random() {
        Ok(encode_random(data.to_vec(), blocksize))
    } else if encodertype == systematic() {
        Ok(encode_systematic(data.to_vec(), blocksize))
    } else {
        Err(Error::BadArg)
    }
}

#[rustler::nif(name = "next")]
fn next<'a>(env: Env<'a>, encoder_res: ResourceArc<EncoderRes>) -> NifResult<Term<'a>> {
    let mut enc = encoder_res.encoder.write().unwrap();
    let droplet = enc.next();

    match droplet {
        Some(drop) => {
            match drop.droptype {
                DropType::Seeded(seed, degree) => {
                    let data = drop.data;
                    let mut binary = OwnedBinary::new(data.len()).unwrap();
                    binary.as_mut_slice().write_all(&data).unwrap();
                    Ok(((seeded(), seed, degree), enc.cnt, Binary::from_owned(binary, env))
                        .encode(env))
                }
                DropType::Edges(es) => {
                    let data = drop.data;
                    let mut binary = OwnedBinary::new(data.len()).unwrap();
                    binary.as_mut_slice().write_all(&data).unwrap();
                    Ok(((edges(), es), Binary::from_owned(binary, env)).encode(env))
                }
            }
        }
        None =>
            Err(Error::BadArg)
    }
}

fn encode_systematic(buf: Vec<u8>, chunk: usize) -> ResourceArc<EncoderRes> {
    let resource = ResourceArc::new(EncoderRes {
        encoder: RwLock::new(FountainEncoder::new(buf, chunk, EncoderType::Systematic))
    });

    resource
}

fn encode_random(buf: Vec<u8>, length: usize) -> ResourceArc<EncoderRes> {
    let resource = ResourceArc::new(EncoderRes {
        encoder: RwLock::new(FountainEncoder::new(buf, length, EncoderType::Random))
    });

    resource
}
