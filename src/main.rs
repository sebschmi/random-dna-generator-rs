use clap::Parser;
use rand::seq::SliceRandom;
use rand::{Rng, rngs::SmallRng, SeedableRng};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use rand::distributions::WeightedIndex;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long)]
    length: usize,

    #[clap(short, long, requires = "subsequence-out")]
    subsequence_length: Option<usize>,

    #[clap(long)]
    sequence_out: PathBuf,

    #[clap(long, requires = "subsequence-length")]
    subsequence_out: Option<PathBuf>,
}

fn main() {
    let options = Cli::parse();

    let sequence = generate(options.length, &mut SmallRng::from_entropy());
    let output = BufWriter::new(File::create(options.sequence_out).unwrap());
    write_fasta_record("random_reference", sequence.as_slice(), output);

    if let Some(subsequence_length) = options.subsequence_length {
        let mut output = BufWriter::new(File::create(options.subsequence_out.unwrap()).unwrap());
        assert!(subsequence_length <= sequence.len());

        let subsequence = &sequence[sequence.len() - subsequence_length..];
        let reverse_complement = reverse_complement(subsequence);

        write_fasta_record("random_contig", subsequence, &mut output);
        write_fasta_record("random_contig_rev", reverse_complement.as_slice(), output);
    }

    println!("Done");
}

fn generate(length: usize, rng: &mut impl Rng) -> Vec<u8> {
    let alphabet = [b'A', b'C', b'G', b'T'];
    let mut result = Vec::new();

    let repetition_weights = (0..100).map(|r| if r == 0 {0.0} else {0.9f32.powi(r - 1)
        + if r <= 2 {
        200.0
    } else if r <= 4 {
        40.0
    } else if r <= 10 {
        1.0
    } else {
        0.0
    }});
    let repetition_distribution = WeightedIndex::new(repetition_weights).unwrap();

    while result.len() < length {
        let character = *alphabet.choose(rng).unwrap();
        let repetitions = rng.sample(&repetition_distribution);

        for _ in 0..repetitions {
            if result.len() >= length {
                break;
            }

            result.push(character);
        }
    }

    result
}

fn reverse_complement(forwards: &[u8]) -> Vec<u8> {
    forwards
        .iter()
        .rev()
        .map(|character| match character {
            b'A' => b'T',
            b'C' => b'G',
            b'G' => b'C',
            b'T' => b'A',
            other => panic!("unexpected: {other}"),
        })
        .collect()
}

fn write_fasta_record(name: &str, sequence: &[u8], mut writer: impl Write) {
    writeln!(writer, ">{name}").unwrap();
    writer.write_all(sequence).unwrap();
    writeln!(writer).unwrap();
}
