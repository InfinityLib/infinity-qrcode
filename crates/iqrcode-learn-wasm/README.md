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

console.log(trace.version(), trace.size(), trace.segments());
console.log(frame.title(), frame.lesson_title());
console.log(frame.matrix());    // Uint8Array, 0/1
console.log(frame.highlight()); // Uint32Array, paires (x, y)

frame.free();
trace.free();
```

Les objets WASM exportés (`LearnTrace`, `LearnFrame`) doivent être libérés avec
`free()` quand ils ne servent plus.
