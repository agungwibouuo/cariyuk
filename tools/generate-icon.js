import fs from 'fs';
import https from 'https';

const API_KEY = process.env.RADJA_API_KEY;
if (!API_KEY) {
  console.error('Set RADJA_API_KEY env var first: $env:RADJA_API_KEY = "sk-..."');
  process.exit(1);
}
const prompt = 'Modern minimalist app icon for "CariYuk", a blazing fast desktop file search application like macOS Spotlight. A stylized luminous magnifying glass focusing on subtle file document sheets, sleek geometry, vibrant indigo and purple gradient squircle with rounded corners, dark mode aesthetic, modern premium glassmorphism, clean vector 3D icon style, centered, high quality, no text';

const payload = JSON.stringify({
  model: 'ag/gemini-3.1-flash-image',
  prompt,
  n: 1,
  size: '1024x1024',
  quality: 'auto',
  background: 'auto',
  image_detail: 'high',
  output_format: 'png'
});

const req = https.request('https://router.radja.id/v1/images/generations', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`,
    'Content-Length': Buffer.byteLength(payload)
  }
}, (res) => {
  let data = '';
  res.on('data', chunk => { data += chunk; });
  res.on('end', () => {
    console.log('Status:', res.statusCode);
    try {
      const json = JSON.parse(data);
      console.log('Response keys:', Object.keys(json));
      if (json.data && json.data.length > 0) {
        const item = json.data[0];
        if (item.b64_json) {
          const buffer = Buffer.from(item.b64_json, 'base64');
          fs.writeFileSync('tools/app-icon.png', buffer);
          console.log('Saved tools/app-icon.png from b64_json, size:', buffer.length, 'bytes');
        } else if (item.url) {
          console.log('Downloading from URL:', item.url);
          download(item.url, 'tools/app-icon.png');
        } else {
          console.log('Unexpected item format:', item);
        }
      } else {
        console.error('Error in API response:', json);
      }
    } catch (e) {
      console.error('Parse error:', e.message, 'Raw response:', data.substring(0, 500));
    }
  });
});

req.on('error', (e) => {
  console.error('Request error:', e);
});

req.write(payload);
req.end();

function download(url, dest) {
  const file = fs.createWriteStream(dest);
  https.get(url, (response) => {
    response.pipe(file);
    file.on('finish', () => {
      file.close(() => console.log('Downloaded successfully to', dest));
    });
  }).on('error', (err) => {
    fs.unlink(dest, () => {});
    console.error('Download error:', err.message);
  });
}
