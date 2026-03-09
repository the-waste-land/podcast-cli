import http.server
import socketserver
import json
import wave
import struct
import io
import urllib.parse

PORT = 8080

def generate_wav_bytes():
    # Generate 1 sec of silence at 16kHz
    num_samples = 16000
    sample_rate = 16000
    nchannels = 1
    sampwidth = 2
    comptype = "NONE"
    compname = "not compressed"

    output = io.BytesIO()
    wav_file = wave.open(output, 'wb')
    wav_file.setparams((nchannels, sampwidth, sample_rate, num_samples, comptype, compname))
    for _ in range(num_samples):
        wav_file.writeframes(struct.pack('h', 0))
    wav_file.close()
    return output.getvalue()

wav_data = generate_wav_bytes()

class MockHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        if path == "/api/1.0/search/byterm":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            data = {
                "feeds": [
                    {
                        "id": 999,
                        "title": "Sam Altman on AI",
                        "url": "http://127.0.0.1:8080/feed.xml"
                    }
                ]
            }
            self.wfile.write(json.dumps(data).encode("utf-8"))
        elif path == "/api/1.0/episodes/byid":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            data = {
                "episode": {
                    "id": 12345,
                    "title": "Sam Altman Interview",
                    "enclosureUrl": "http://127.0.0.1:8080/audio.wav"
                }
            }
            self.wfile.write(json.dumps(data).encode("utf-8"))
        elif path == "/audio.wav":
            self.send_response(200)
            self.send_header("Content-type", "audio/wav")
            self.send_header("Content-Length", str(len(wav_data)))
            self.end_headers()
            self.wfile.write(wav_data)
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not found")

with socketserver.TCPServer(("127.0.0.1", PORT), MockHandler) as httpd:
    print(f"Serving at port {PORT}")
    httpd.serve_forever()
