# iqrcode-learn-wasm

Bindings WebAssembly pour `iqrcode-learn`.

La crate expose la trace de construction et les valeurs calculées. Elle ne
contient pas d’interface web ni de traductions : un frontend séparé peut les
fournir à partir de cette API.

## Build

```bash
cd crates/iqrcode-learn-wasm
wasm-pack build --target web
```

Le paquet est généré dans `pkg/`.

## API JS

```js
import init, { LearnTrace } from './pkg/iqrcode_learn_wasm.js';

await init();

const trace = new LearnTrace('HELLO WORLD', 'M', undefined, 0, 300, true);
const frame = trace.frame(0);

console.log(trace.version(), trace.size(), trace.data_modules(), trace.segments());
console.log(frame.phase());       // "empty", "data_placement", ...
console.log(frame.bit_index());   // number | undefined
console.log(frame.mask());        // number | undefined
console.log(frame.penalty());     // number | undefined
console.log(frame.matrix());      // Uint8Array, 0/1
console.log(frame.highlight());   // Uint32Array, paires (x, y)

frame.free();
trace.free();
```

L’interface et les traductions vivent dans le futur dépôt web, pas ici.

Les objets WASM exportés (`LearnTrace`, `LearnFrame`) doivent être libérés avec
`free()` quand ils ne servent plus.
