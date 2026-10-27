# iqrcode-learn-wasm

Version WebAssembly de `iqrcode-learn`.

## Build

```bash
cd crates/iqrcode-learn-wasm
wasm-pack build --target web --out-dir web/pkg
```

Puis :

```bash
cd web
python3 -m http.server 8080
```

Ouvrir <http://localhost:8080>.

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

Les textes et traductions ne sont pas dans le WASM. Ils sont dans :

```text
web/i18n/fr.js
web/i18n/en.js
```

L'UI choisit la langue et associe `frame.phase()` à la bonne explication.

Les objets WASM exportés (`LearnTrace`, `LearnFrame`) doivent être libérés avec
`free()` quand ils ne servent plus.
