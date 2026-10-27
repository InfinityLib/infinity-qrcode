const levels = {
  Low: "Low",
  Medium: "Moyen",
  Quartile: "Quartile",
  High: "Élevé",
};

export const fr = {
  levels,
  ui: {
    title: "iqrcode-learn",
    subtitle:
      "Construction pas à pas d'un QR Code, avec les explications de chaque étape.",
    data: "Données",
    level: "Niveau",
    version: "Version",
    language: "Langue",
    auto: "auto",
    regenerate: "Régénérer",
    previous: "Précédent",
    next: "Suivant",
    play: "Play",
    pause: "Pause",
    speed: "Vitesse",
    pipeline: "Le pipeline avant la matrice",
    formula: "Formule",
    errorPrefix: "Erreur : ",
    meta: (trace) =>
      `Version ${trace.version()} · ${levels[trace.level()] ?? trace.level()} · ` +
      `${trace.size()}×${trace.size()} · ${trace.data_modules()} modules data · ` +
      `${trace.data_codewords()} data · ${trace.ecc_per_block()} ECC/bloc · ` +
      `${trace.blocks()} bloc(s) · ${trace.segments()}`,
    mask: (frame) =>
      frame.mask() === undefined
        ? ""
        : `Masque ${frame.mask()} · pénalité ${frame.penalty() ?? "—"}`,
    frameTitle: (title, index, total) => `${title} — ${index + 1}/${total}`,
  },
  phases: {
    empty: {
      title: "Matrice vide",
      description: (frame, trace) =>
        `Version ${trace.version()} : matrice de ${trace.size()}×${trace.size()} modules.`,
      formula: "taille = 17 + 4v",
    },
    finders: {
      title: "Finder patterns",
      description:
        "Trois carrés 7×7 permettent au scanner de repérer le QR et son orientation. Chaque finder est entouré d'un séparateur clair.",
      formula: "",
    },
    timing: {
      title: "Timing patterns",
      description:
        "La ligne 6 et la colonne 6 alternent noir et blanc. Elles donnent une référence de taille au décodeur.",
      formula: "module(i, 6) = i mod 2",
    },
    alignment: {
      title: "Alignment patterns",
      description:
        "Des motifs 5×5 sont placés sur une grille. Ils aident le scanner à corriger la perspective et les déformations.",
      formula: "n = v/7 + 2 ; step = (8v + 3n + 5)/(4n − 4) × 2",
    },
    version_info: {
      title: "Version information",
      description:
        "À partir de la version 7, 18 bits indiquent la version. Ils sont protégés par un code BCH(18,6) et placés en deux copies.",
      formula:
        "version_info = (version << 12) | BCH_12(version), générateur 0x1F25",
    },
    format_reserved: {
      title: "Format information réservée",
      description:
        "Deux copies de 15 bits accueilleront le niveau de correction et le masque. Le dark module est toujours noir.",
      formula: "",
    },
    data_placement: {
      title: "Placement des données",
      description: (frame, trace) => {
        const highlight = frame.highlight();
        const x = highlight[0] ?? 0;
        const y = highlight[1] ?? 0;
        return `Bit ${(frame.bit_index() ?? 0) + 1} / ${trace.data_modules()} — module (${x}, ${y})`;
      },
      formula: "paires de colonnes, droite → gauche, zigzag, colonne 6 sautée",
    },
    mask_candidate: {
      title: "Candidat de masque",
      description: (frame) =>
        `Masque ${frame.mask()} — pénalité ${frame.penalty() ?? "—"}`,
      formula: "N1=3, N2=3, N3=40, N4=10 par tranche de 5 %",
    },
    mask_chosen: {
      title: "Masque retenu",
      description: (frame) =>
        `Le masque ${frame.mask()} est appliqué aux modules de données uniquement.`,
      formula: "masque = argmin(score)",
    },
    format_info: {
      title: "Format information",
      description:
        "Les 5 bits utiles (2 pour le niveau, 3 pour le masque) sont protégés par un BCH(15,5), puis XORés avec 0x5412.",
      formula:
        "format = ((data << 10) | BCH_10(data)) XOR 0x5412, générateur 0x537",
    },
    final: {
      title: "QR Code final",
      description: (frame, trace) =>
        `Version ${trace.version()}, ${levels[trace.level()] ?? trace.level()}, masque ${frame.mask() ?? trace.mask()}.`,
      formula: "",
    },
    unknown: {
      title: "Étape inconnue",
      description: "",
      formula: "",
    },
  },
  pipeline: [
    {
      title: "Segmentation",
      explanation:
        "Le texte est découpé en segments Numeric, Alphanumeric ou Byte. Chaque segment a un en-tête et un compteur, mais un mode plus compact peut faire économiser beaucoup de bits.",
      formula: "coût = 4 bits de mode + bits du compteur + bits des données",
    },
    {
      title: "Flux de bits",
      explanation:
        "On ajoute un terminator, on aligne sur l'octet, puis on remplit la capacité avec les octets de padding 0xEC et 0x11 en alternance.",
      formula: "capacité = 8 × data_codewords",
    },
    {
      title: "Codewords",
      explanation:
        "Les bits sont regroupés en octets, puis répartis en blocs selon la version et le niveau de correction.",
      formula: "data_codewords = total_codewords − ecc_par_bloc × blocs",
    },
    {
      title: "Reed-Solomon",
      explanation:
        "Chaque bloc reçoit des codewords de correction. Le polynôme générateur a pour racines α^0 … α^(t−1), donc le codeword complet est divisible par ce polynôme.",
      formula: "g(x) = ∏(x − α^i),  c(x) = m(x)·x^t + r(x)",
    },
    {
      title: "Entrelacement",
      explanation:
        "Les codewords de données puis d'ECC sont entrelacés entre les blocs. Une rayure sur le QR est ainsi répartie sur plusieurs blocs.",
      formula: "data[0][0], data[1][0], … puis ecc[0][0], ecc[1][0], …",
    },
  ],
};
