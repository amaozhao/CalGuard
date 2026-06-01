import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const root = process.cwd();
const iconPath = path.join(root, 'apps/desktop/src-tauri/icons/icon.png');
const publicIconPath = path.join(root, 'apps/desktop/public/icon.png');
const size = 1024;
const scale = 3;
const canvasSize = size * scale;
const pixels = new Uint8ClampedArray(canvasSize * canvasSize * 4);

drawIcon();

const data = downsample();
const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk('IHDR', header(size, size)),
  chunk('IDAT', zlib.deflateSync(data, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);

fs.mkdirSync(path.dirname(iconPath), { recursive: true });
fs.mkdirSync(path.dirname(publicIconPath), { recursive: true });
fs.writeFileSync(iconPath, png);
fs.writeFileSync(publicIconPath, png);

function drawIcon() {
  const s = scale;
  roundedRect(70 * s, 70 * s, 884 * s, 884 * s, 210 * s, [3, 11, 26, 255]);
  roundedRect(104 * s, 104 * s, 816 * s, 816 * s, 178 * s, [8, 48, 73, 255]);
  radialGlow(730 * s, 230 * s, 430 * s, [20, 184, 166, 126]);
  radialGlow(250 * s, 780 * s, 470 * s, [37, 99, 235, 128]);
  thickArc(512 * s, 512 * s, 354 * s, 205, 516, 42 * s, [250, 204, 21, 255]);

  // Oversized CG mark: intentionally simple so the dock icon is recognizable at 32px.
  thickArc(366 * s, 510 * s, 190 * s, 54, 306, 86 * s, [245, 255, 252, 255]);
  thickArc(664 * s, 510 * s, 190 * s, 30, 336, 86 * s, [245, 255, 252, 255]);
  thickLine(664 * s, 512 * s, 818 * s, 512 * s, 43 * s, [245, 255, 252, 255]);
  thickLine(818 * s, 512 * s, 818 * s, 630 * s, 43 * s, [245, 255, 252, 255]);

  roundedRect(318 * s, 710 * s, 390 * s, 92 * s, 46 * s, [13, 148, 136, 255]);
  thickLine(410 * s, 754 * s, 486 * s, 818 * s, 32 * s, [250, 204, 21, 255]);
  thickLine(486 * s, 818 * s, 642 * s, 662 * s, 32 * s, [250, 204, 21, 255]);
}

function roundedRect(x, y, width, height, radius, color) {
  const x2 = x + width;
  const y2 = y + height;
  for (let yy = Math.floor(y); yy < Math.ceil(y2); yy += 1) {
    for (let xx = Math.floor(x); xx < Math.ceil(x2); xx += 1) {
      const cx = clamp(xx, x + radius, x2 - radius);
      const cy = clamp(yy, y + radius, y2 - radius);
      if ((xx - cx) ** 2 + (yy - cy) ** 2 <= radius ** 2) {
        blendPixel(xx, yy, color);
      }
    }
  }
}

function circle(cx, cy, radius, color) {
  const r2 = radius ** 2;
  for (let y = Math.floor(cy - radius); y <= Math.ceil(cy + radius); y += 1) {
    for (let x = Math.floor(cx - radius); x <= Math.ceil(cx + radius); x += 1) {
      if ((x - cx) ** 2 + (y - cy) ** 2 <= r2) {
        blendPixel(x, y, color);
      }
    }
  }
}

function radialGlow(cx, cy, radius, color) {
  const minX = Math.floor(cx - radius);
  const maxX = Math.ceil(cx + radius);
  const minY = Math.floor(cy - radius);
  const maxY = Math.ceil(cy + radius);
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const distance = Math.hypot(x - cx, y - cy) / radius;
      if (distance <= 1) {
        const alpha = Math.round(color[3] * (1 - distance) ** 2);
        blendPixel(x, y, [color[0], color[1], color[2], alpha]);
      }
    }
  }
}

function thickArc(cx, cy, radius, startDeg, endDeg, width, color) {
  const start = (startDeg * Math.PI) / 180;
  const end = (endDeg * Math.PI) / 180;
  const step = 1 / radius;
  let previous = null;
  for (let angle = start; angle <= end; angle += step) {
    const point = [cx + Math.cos(angle) * radius, cy + Math.sin(angle) * radius];
    if (previous) {
      thickLine(previous[0], previous[1], point[0], point[1], width / 2, color);
    }
    previous = point;
  }
}

function thickLine(x0, y0, x1, y1, radius, color) {
  const minX = Math.floor(Math.min(x0, x1) - radius);
  const maxX = Math.ceil(Math.max(x0, x1) + radius);
  const minY = Math.floor(Math.min(y0, y1) - radius);
  const maxY = Math.ceil(Math.max(y0, y1) + radius);
  const dx = x1 - x0;
  const dy = y1 - y0;
  const lenSq = dx * dx + dy * dy || 1;
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const t = clamp(((x - x0) * dx + (y - y0) * dy) / lenSq, 0, 1);
      const px = x0 + t * dx;
      const py = y0 + t * dy;
      if ((x - px) ** 2 + (y - py) ** 2 <= radius ** 2) {
        blendPixel(x, y, color);
      }
    }
  }
}

function polygon(points, color) {
  const minY = Math.floor(Math.min(...points.map((point) => point[1])));
  const maxY = Math.ceil(Math.max(...points.map((point) => point[1])));
  for (let y = minY; y <= maxY; y += 1) {
    const intersections = [];
    for (let i = 0; i < points.length; i += 1) {
      const [x1, y1] = points[i];
      const [x2, y2] = points[(i + 1) % points.length];
      if ((y1 <= y && y2 > y) || (y2 <= y && y1 > y)) {
        intersections.push(x1 + ((y - y1) * (x2 - x1)) / (y2 - y1));
      }
    }
    intersections.sort((a, b) => a - b);
    for (let i = 0; i < intersections.length; i += 2) {
      for (let x = Math.ceil(intersections[i]); x <= Math.floor(intersections[i + 1]); x += 1) {
        blendPixel(x, y, color);
      }
    }
  }
}

function blendPixel(x, y, color) {
  if (x < 0 || x >= canvasSize || y < 0 || y >= canvasSize || color[3] <= 0) return;
  const idx = (Math.floor(y) * canvasSize + Math.floor(x)) * 4;
  const alpha = color[3] / 255;
  const inverse = 1 - alpha;
  pixels[idx] = Math.round(color[0] * alpha + pixels[idx] * inverse);
  pixels[idx + 1] = Math.round(color[1] * alpha + pixels[idx + 1] * inverse);
  pixels[idx + 2] = Math.round(color[2] * alpha + pixels[idx + 2] * inverse);
  pixels[idx + 3] = Math.round(255 * alpha + pixels[idx + 3] * inverse);
}

function downsample() {
  const rowStride = size * 4 + 1;
  const out = Buffer.alloc(rowStride * size);
  const sampleCount = scale * scale;
  for (let y = 0; y < size; y += 1) {
    const row = y * rowStride;
    out[row] = 0;
    for (let x = 0; x < size; x += 1) {
      const channels = [0, 0, 0, 0];
      for (let sy = 0; sy < scale; sy += 1) {
        for (let sx = 0; sx < scale; sx += 1) {
          const idx = ((y * scale + sy) * canvasSize + (x * scale + sx)) * 4;
          channels[0] += pixels[idx];
          channels[1] += pixels[idx + 1];
          channels[2] += pixels[idx + 2];
          channels[3] += pixels[idx + 3];
        }
      }
      const outIdx = row + 1 + x * 4;
      out[outIdx] = Math.round(channels[0] / sampleCount);
      out[outIdx + 1] = Math.round(channels[1] / sampleCount);
      out[outIdx + 2] = Math.round(channels[2] / sampleCount);
      out[outIdx + 3] = Math.round(channels[3] / sampleCount);
    }
  }
  return out;
}

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function header(width, height) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  return ihdr;
}

function chunk(type, payload) {
  const typeBuffer = Buffer.from(type);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, payload])));
  return Buffer.concat([length, typeBuffer, payload, crc]);
}

function crc32(buffer) {
  let crc = ~0;
  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
    }
  }
  return ~crc >>> 0;
}
