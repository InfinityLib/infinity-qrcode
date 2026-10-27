const levels = {
  Low: "Low",
  Medium: "Medium",
  Quartile: "Quartile",
  High: "High",
};

export const en = {
  levels,
  ui: {
    title: "iqrcode-learn",
    subtitle: "Step-by-step construction of a QR Code, with an explanation for each stage.",
    data: "Data",
    level: "Level",
    version: "Version",
    language: "Language",
    auto: "auto",
    regenerate: "Regenerate",
    previous: "Previous",
    next: "Next",
    play: "Play",
    pause: "Pause",
    speed: "Speed",
    pipeline: "The pipeline before the matrix",
    formula: "Formula",
    errorPrefix: "Error: ",
    meta: (trace) =>
      `Version ${trace.version()} · ${trace.level()} · ` +
      `${trace.size()}×${trace.size()} · ${trace.data_modules()} data modules · ` +
      `${trace.data_codewords()} data · ${trace.ecc_per_block()} ECC/block · ` +
      `${trace.blocks()} block(s) · ${trace.segments()}`,
    mask: (frame) =>
      frame.mask() === undefined
        ? ""
        : `Mask ${frame.mask()} · penalty ${frame.penalty() ?? "—"}`,
    frameTitle: (title, index, total) => `${title} — ${index + 1}/${total}`,
  },
  phases: {
    empty: {
      title: "Empty matrix",
      description: (frame, trace) =>
        `Version ${trace.version()}: a ${trace.size()}×${trace.size()} module matrix.`,
      formula: "size = 17 + 4v",
    },
    finders: {
      title: "Finder patterns",
      description:
        "Three 7×7 squares let the scanner locate the QR Code and its orientation. Each finder is surrounded by a light separator.",
      formula: "",
    },
    timing: {
      title: "Timing patterns",
      description:
        "Row 6 and column 6 alternate black and white. They give the decoder a size reference.",
      formula: "module(i, 6) = i mod 2",
    },
    alignment: {
      title: "Alignment patterns",
      description:
        "5×5 patterns are placed on a grid. They help the scanner correct perspective and distortion.",
      formula: "n = v/7 + 2 ; step = (8v + 3n + 5)/(4n − 4) × 2",
    },
    version_info: {
      title: "Version information",
      description:
        "From version 7 onward, 18 bits encode the version. They are protected by a BCH(18,6) code and placed in two copies.",
      formula:
        "version_info = (version << 12) | BCH_12(version), generator 0x1F25",
    },
    format_reserved: {
      title: "Reserved format information",
      description:
        "Two 15-bit copies will hold the correction level and the mask. The dark module is always black.",
      formula: "",
    },
    data_placement: {
      title: "Data placement",
      description: (frame, trace) => {
        const highlight = frame.highlight();
        const x = highlight[0] ?? 0;
        const y = highlight[1] ?? 0;
        return `Bit ${(frame.bit_index() ?? 0) + 1} / ${trace.data_modules()} — module (${x}, ${y})`;
      },
      formula: "two-column pairs, right → left, zigzag, column 6 skipped",
    },
    mask_candidate: {
      title: "Mask candidate",
      description: (frame) =>
        `Mask ${frame.mask()} — penalty ${frame.penalty() ?? "—"}`,
      formula: "N1=3, N2=3, N3=40, N4=10 per 5% step",
    },
    mask_chosen: {
      title: "Chosen mask",
      description: (frame) =>
        `Mask ${frame.mask()} is applied to data modules only.`,
      formula: "mask = argmin(score)",
    },
    format_info: {
      title: "Format information",
      description:
        "The 5 payload bits (2 for the level, 3 for the mask) are protected by BCH(15,5), then XORed with 0x5412.",
      formula:
        "format = ((data << 10) | BCH_10(data)) XOR 0x5412, generator 0x537",
    },
    final: {
      title: "Final QR Code",
      description: (frame, trace) =>
        `Version ${trace.version()}, ${trace.level()}, mask ${frame.mask() ?? trace.mask()}.`,
      formula: "",
    },
    unknown: {
      title: "Unknown stage",
      description: "",
      formula: "",
    },
  },
  pipeline: [
    {
      title: "Segmentation",
      explanation:
        "The text is split into Numeric, Alphanumeric or Byte segments. Each segment has a header and a counter, but a more compact mode can save many bits.",
      formula: "cost = 4 mode bits + counter bits + data bits",
    },
    {
      title: "Bit stream",
      explanation:
        "A terminator is added, the stream is byte-aligned, then the capacity is filled with alternating 0xEC and 0x11 padding bytes.",
      formula: "capacity = 8 × data_codewords",
    },
    {
      title: "Codewords",
      explanation:
        "Bits are grouped into bytes, then split into blocks according to the version and correction level.",
      formula: "data_codewords = total_codewords − ecc_per_block × blocks",
    },
    {
      title: "Reed-Solomon",
      explanation:
        "Each block gets correction codewords. The generator polynomial has roots α^0 … α^(t−1), so the full codeword is divisible by it.",
      formula: "g(x) = ∏(x − α^i),  c(x) = m(x)·x^t + r(x)",
    },
    {
      title: "Interleaving",
      explanation:
        "Data codewords and ECC codewords are interleaved across blocks. A scratch on the QR Code is spread over several blocks.",
      formula: "data[0][0], data[1][0], … then ecc[0][0], ecc[1][0], …",
    },
  ],
};
