use std::cmp::min;
use std::ops::Range;

use metagene::*;

use fp_framing::framing::*;

pub struct FramingStats {
    frame_length: Frame<LenProfile<usize>>,
    around_start: Metagene<LenProfile<usize>>,
    around_end: Metagene<LenProfile<usize>>,
    align_stats: AlignStats,
}

#[allow(dead_code)]
impl FramingStats {
    pub fn new(lengths: &Range<usize>, flanking: &Range<isize>) -> Self {
        let len_profile = LenProfile::new_with_default(lengths.start, lengths.end);

        let flanking_len = flanking.end as usize - min(flanking.end, flanking.start) as usize;

        FramingStats {
            frame_length: Frame::new(len_profile.clone()),
            around_start: Metagene::new(flanking.start, flanking_len, len_profile.clone()),
            around_end: Metagene::new(flanking.start, flanking_len, len_profile.clone()),
            align_stats: AlignStats::new(),
        }
    }

    pub fn frame_length(&self) -> &Frame<LenProfile<usize>> {
        &self.frame_length
    }
    pub fn around_start(&self) -> &Metagene<LenProfile<usize>> {
        &self.around_start
    }
    pub fn around_end(&self) -> &Metagene<LenProfile<usize>> {
        &self.around_end
    }
    pub fn align_stats(&self) -> &AlignStats {
        &self.align_stats
    }

    fn align_stats_mut(&mut self) -> &mut AlignStats {
        &mut self.align_stats
    }

    pub fn tally_frame_length(&mut self, frame: isize, fp_length: usize) {
        *self.frame_length.get_mut(frame).get_mut(fp_length) += 1
    }

    pub fn tally_around_start(&mut self, start_offset: isize, fp_length: usize) {
        self.around_start.get_mut(start_offset).map(|vs_start| *vs_start.get_mut(fp_length) += 1);
    }

    pub fn tally_around_end(&mut self, end_offset: isize, fp_length: usize) {
        self.around_end.get_mut(end_offset).map(|vs_end| *vs_end.get_mut(fp_length) += 1);
    }

    pub fn tally_bam_frame(&mut self, bam_frame: &BamFrameResult) {
        self.align_stats_mut().tally_bam_frame(bam_frame);

        match bam_frame {
            BamFrameResult::Fp(FpFrameResult::Gene(GeneFrameResult::Good(gene_frame))) => {
                gene_frame
                    .frame()
                    .map(|fr| self.tally_frame_length(fr as isize, gene_frame.fp_length()));
                gene_frame.vs_cds_start().map(|start_offset| self.tally_around_start(start_offset, gene_frame.fp_length()));
                gene_frame.vs_cds_end().map(|end_offset| self.tally_around_end(end_offset, gene_frame.fp_length()));
            }
            _ => (),
        };
    }
}

pub struct AnnotStats {
    no_gene: usize,
    noncoding: usize,
    noncoding_overlap: usize,
    multi_coding: usize,
    incompatible: usize,
    ambig: usize,
    good: usize,
}

#[allow(dead_code)]
impl AnnotStats {
    pub fn new() -> Self {
        AnnotStats {
            no_gene: 0,
            noncoding: 0,
            noncoding_overlap: 0,
            multi_coding: 0,
            incompatible: 0,
            ambig: 0,
            good: 0,
        }
    }

    pub fn no_gene(&self) -> usize {
        self.no_gene
    }
    pub fn noncoding(&self) -> usize {
        self.noncoding
    }
    pub fn noncoding_overlap(&self) -> usize {
        self.noncoding_overlap
    }
    pub fn multi_coding(&self) -> usize {
        self.multi_coding
    }
    pub fn incompatible(&self) -> usize {
        self.incompatible
    }
    pub fn ambig(&self) -> usize {
        self.ambig
    }
    pub fn good(&self) -> usize {
        self.good
    }

    pub fn tally_fp_frame(&mut self, fp_frame: &FpFrameResult) {
        match fp_frame {
            FpFrameResult::NoGene => self.no_gene += 1,
            FpFrameResult::NoncodingOnly => self.noncoding += 1,
            FpFrameResult::NoncodingOverlap => self.noncoding_overlap += 1,
            FpFrameResult::MultiCoding => self.multi_coding += 1,
            FpFrameResult::Gene(GeneFrameResult::NoCompatible) => self.incompatible += 1,
            FpFrameResult::Gene(GeneFrameResult::Ambig) => self.ambig += 1,
            FpFrameResult::Gene(GeneFrameResult::Good(_)) => self.good += 1,
        }
    }

    pub fn bad_total(&self) -> usize {
        self.no_gene
            + self.noncoding
            + self.noncoding_overlap
            + self.multi_coding
            + self.incompatible
            + self.ambig
    }

    pub fn total(&self) -> usize {
        self.bad_total() + self.good
    }

    pub fn table(&self, align_ttl: f64) -> String {
        let mut tbl = String::new();

        let ttl = self.total() as f64;

        tbl += &format!(
            "\tNoGene\t{}\t{:.4}\t{:.4}\n",
            self.no_gene(),
            self.no_gene() as f64 / align_ttl,
            self.no_gene() as f64 / ttl
        );
        tbl += &format!(
            "\tNoncodingOnly\t{}\t{:.4}\t{:.4}\n",
            self.noncoding(),
            self.noncoding() as f64 / align_ttl,
            self.noncoding() as f64 / ttl
        );
        tbl += &format!(
            "\tNoncodingOverlap\t{}\t{:.4}\t{:.4}\n",
            self.noncoding_overlap(),
            self.noncoding_overlap() as f64 / align_ttl,
            self.noncoding_overlap() as f64 / ttl
        );
        tbl += &format!(
            "\tMultiCoding\t{}\t{:.4}\t{:.4}\n",
            self.multi_coding(),
            self.multi_coding() as f64 / align_ttl,
            self.multi_coding() as f64 / ttl
        );
        tbl += &format!(
            "\tNoCompatible\t{}\t{:.4}\t{:.4}\n",
            self.incompatible(),
            self.incompatible() as f64 / align_ttl,
            self.incompatible() as f64 / ttl
        );
        tbl += &format!(
            "\tAmbigFrame\t{}\t{:.4}\t{:.4}\n",
            self.ambig(),
            self.ambig() as f64 / align_ttl,
            self.ambig() as f64 / ttl
        );
        tbl += &format!(
            "BadAnnotation\t{}\t{:.4}\t{:.4}\n",
            self.bad_total(),
            self.bad_total() as f64 / align_ttl,
            self.bad_total() as f64 / ttl
        );
        tbl += &format!(
            "GoodAnnotation\t{}\t{:.4}\t{:.4}\n",
            self.good(),
            self.good() as f64 / align_ttl,
            self.good() as f64 / ttl
        );

        tbl
    }
}

pub struct AlignStats {
    unmapped: usize,
    short: usize,
    long: usize,
    multi_hit: usize,
    annot_stats: AnnotStats,
}

impl AlignStats {
    pub fn new() -> Self {
        AlignStats {
            unmapped: 0,
            short: 0,
            long: 0,
            multi_hit: 0,
            annot_stats: AnnotStats::new(),
        }
    }

    pub fn unmapped(&self) -> usize {
        self.unmapped
    }
    pub fn short(&self) -> usize {
        self.short
    }
    pub fn long(&self) -> usize {
        self.long
    }
    pub fn multi_hit(&self) -> usize {
        self.multi_hit
    }

    pub fn tally_bam_frame(&mut self, bam_frame: &BamFrameResult) {
        match bam_frame {
            BamFrameResult::NoHit => self.unmapped += 1,
            BamFrameResult::MultiHit => self.multi_hit += 1,
            BamFrameResult::TooShort => self.short += 1,
            BamFrameResult::TooLong => self.long += 1,
            BamFrameResult::Fp(ffr) => self.annot_stats.tally_fp_frame(ffr),
        }
    }

    #[allow(dead_code)]
    pub fn annot_stats_mut(&mut self) -> &mut AnnotStats {
        &mut self.annot_stats
    }

    pub fn total(&self) -> usize {
        self.bad_total() + self.good_total()
    }

    pub fn bad_total(&self) -> usize {
        self.unmapped + self.short + self.long + self.multi_hit
    }

    pub fn good_total(&self) -> usize {
        self.annot_stats.total()
    }

    pub fn table(&self) -> String {
        let mut tbl = String::new();

        let ttl = self.total() as f64;

        tbl += &format!("TOTAL\t\t{}\n", self.total());
        tbl += &format!(
            "\tBamTooShort\t{}\t{:.04}\n",
            self.short(),
            self.short() as f64 / ttl
        );
        tbl += &format!(
            "\tBamTooLong\t{}\t{:.04}\n",
            self.long(),
            self.long() as f64 / ttl
        );
        tbl += &format!(
            "\tBamNoHit\t{}\t{:.04}\n",
            self.unmapped(),
            self.unmapped() as f64 / ttl
        );
        tbl += &format!(
            "\tBamMultiHit\t{}\t{:.04}\n",
            self.multi_hit(),
            self.multi_hit() as f64 / ttl
        );
        tbl += &format!(
            "BadAlignment\t\t{}\t{:.04}\n",
            self.bad_total(),
            self.bad_total() as f64 / ttl
        );
        tbl += &format!(
            "GoodAlignment\t\t{}\t{:.04}\n",
            self.good_total(),
            self.good_total() as f64 / ttl
        );
        tbl += &self.annot_stats.table(ttl);

        // Start, End = # counted in start, end metagene
        // Body = # counted in body framing analysis
        // Not mutually exclusive

        tbl
    }
}
