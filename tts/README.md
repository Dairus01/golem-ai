# golem-tts

WebAssembly Components providing a unified API for various Text-to-Speech (TTS) providers.

## Versions

Each TTS provider has two versions: **Default** (with Golem-specific durability features) and **Portable** (no Golem dependencies).

| Name                         | Description                                                                                |
|------------------------------|--------------------------------------------------------------------------------------------|
| `golem-tts-elevenlabs.wasm`  | TTS implementation for ElevenLabs, with Golem durability features                          |
| `golem-tts-polly.wasm`       | TTS implementation for AWS Polly, with Golem durability features                           |
| `golem-tts-google.wasm`      | TTS implementation for Google Cloud Text-to-Speech, with Golem durability features         |
| `golem-tts-deepgram.wasm`    | TTS implementation for Deepgram Aura, with Golem durability features                        |
| `golem-tts-elevenlabs-portable.wasm` | Portable ElevenLabs implementation                                                  |
| `golem-tts-polly-portable.wasm`      | Portable AWS Polly implementation                                                   |
| `golem-tts-google-portable.wasm`     | Portable Google Cloud Text-to-Speech implementation                                 |
| `golem-tts-deepgram-portable.wasm`   | Portable Deepgram Aura implementation                                              |

Every component exports the same `golem:tts` interface, [defined here](tts/wit/golem-tts.wit).

## Environment Variables

Common configuration:
- `TTS_PROVIDER_ENDPOINT` - Custom endpoint URL override
- `TTS_PROVIDER_TIMEOUT` - Request timeout in seconds (default: 30)
- `TTS_PROVIDER_MAX_RETRIES` - Maximum retry attempts (default: 3)
- `TTS_PROVIDER_LOG_LEVEL` - Logging verbosity (debug, info, warn, error)

Provider-specific:
- **ElevenLabs**: `ELEVENLABS_API_KEY`, `ELEVENLABS_MODEL_VERSION`
- **AWS Polly**: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`, `AWS_SESSION_TOKEN`
- **Google Cloud**: `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_CLOUD_PROJECT`, or (`GOOGLE_CLIENT_EMAIL`, `GOOGLE_PRIVATE_KEY`)
- **Deepgram**: `DEEPGRAM_API_KEY`, `DEEPGRAM_API_VERSION`

## Examples

See the [test application](../test/tts/components-rust/test-tts/src/lib.rs) for basic usage.

### Running the examples

Start a Golem instance, then:

```bash
cd ../test/tts
golem build --preset elevenlabs-debug
golem deploy --preset elevenlabs-debug
```
