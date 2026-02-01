use std::{cell::RefCell, marker::PhantomData};

use crate::golem::tts::advanced::{AudioSample, LongFormResult, VoiceDesignParams};
use crate::golem::tts::streaming::{
    Guest as StreamingGuest, GuestSynthesisStream, GuestVoiceConversionStream, StreamStatus,
    SynthesisOptions,
};
use crate::golem::tts::synthesis::{
    Guest as SynthesisGuest, SynthesisOptions as WitSynthesisOptions, ValidationResult,
};
use crate::golem::tts::types::{AudioChunk, SynthesisResult, TextInput, TimingInfo, TtsError};
use crate::golem::tts::voices::{Guest as VoicesGuest, GuestVoiceResults, VoiceFilter, VoiceInfo};
use crate::guest::{StreamRequest, SynthesisRequest, TtsGuest};
use crate::LOGGING_STATE;

pub struct DurableTts<Impl> {
    phantom: PhantomData<Impl>,
}

pub trait ExtendedGuest: TtsGuest + 'static {}

pub struct DurableVoiceResults {
    voices: RefCell<Vec<VoiceInfo>>,
}

impl DurableVoiceResults {
    pub fn new(voices: Vec<VoiceInfo>) -> Self {
        Self {
            voices: RefCell::new(voices),
        }
    }
}

impl GuestVoiceResults for DurableVoiceResults {
    fn has_more(&self) -> bool {
        !self.voices.borrow().is_empty()
    }

    fn get_next(&self) -> Result<Vec<VoiceInfo>, TtsError> {
        Ok(self.voices.borrow_mut().drain(..).collect())
    }

    fn get_total_count(&self) -> Option<u32> {
        Some(self.voices.borrow().len() as u32)
    }
}

#[cfg(not(feature = "durability"))]
mod passthrough_impl {
    use super::*;

    impl<Impl: ExtendedGuest> VoicesGuest for DurableTts<Impl> {
        type Voice = crate::voices::VoiceResource;
        type VoiceResults = DurableVoiceResults;

        fn list_voices(
            filter: Option<VoiceFilter>,
        ) -> Result<crate::golem::tts::voices::VoiceResults, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let voices = Impl::list_voices(filter)?;
            Ok(crate::golem::tts::voices::VoiceResults::new(DurableVoiceResults::new(
                voices,
            )))
        }

        fn get_voice(voice_id: String) -> Result<crate::golem::tts::voices::Voice, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let info = Impl::get_voice(voice_id)?;
            Ok(crate::golem::tts::voices::Voice::new(
                crate::voices::VoiceResource::from_info(&info),
            ))
        }

        fn search_voices(
            query: String,
            filter: Option<VoiceFilter>,
        ) -> Result<crate::golem::tts::voices::VoiceResults, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let voices = Impl::search_voices(query, filter)?;
            Ok(crate::golem::tts::voices::VoiceResults::new(DurableVoiceResults::new(
                voices,
            )))
        }

        fn list_languages() -> Result<Vec<String>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::list_languages()
        }
    }

    impl<Impl: ExtendedGuest> SynthesisGuest for DurableTts<Impl> {
        fn synthesize(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
            options: Option<WitSynthesisOptions>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::synthesize(SynthesisRequest {
                input,
                voice_id: voice.get_id(),
                options,
            })
        }

        fn synthesize_batch(
            inputs: Vec<TextInput>,
            voice: crate::golem::tts::voices::Voice,
            options: Option<WitSynthesisOptions>,
        ) -> Result<Vec<SynthesisResult>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::synthesize_batch(
                inputs
                    .into_iter()
                    .map(|input| SynthesisRequest {
                        input,
                        voice_id: voice.get_id(),
                        options: options.clone(),
                    })
                    .collect(),
            )
        }

        fn get_timing_marks(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
        ) -> Result<Vec<TimingInfo>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::get_timing_marks(input, voice.get_id())
        }

        fn validate_input(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
        ) -> Result<ValidationResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::validate_input(input, voice.get_id())
        }
    }

    impl<Impl: ExtendedGuest> StreamingGuest for DurableTts<Impl> {
        type SynthesisStream = DurableSynthesisStream<Impl>;
        type VoiceConversionStream = DurableVoiceConversionStream<Impl>;

        fn create_stream(
            voice: crate::golem::tts::voices::Voice,
            options: Option<SynthesisOptions>,
        ) -> Result<Self::SynthesisStream, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::create_stream(StreamRequest {
                voice_id: voice.get_id(),
                options,
            })
        }

        fn create_voice_conversion_stream(
            target_voice: crate::golem::tts::voices::Voice,
            options: Option<SynthesisOptions>,
        ) -> Result<Self::VoiceConversionStream, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::create_voice_conversion_stream(StreamRequest {
                voice_id: target_voice.get_id(),
                options,
            })
        }
    }

    impl<Impl: ExtendedGuest> crate::golem::tts::advanced::Guest for DurableTts<Impl> {
        fn create_voice_clone(
            name: String,
            audio_samples: Vec<AudioSample>,
            description: Option<String>,
        ) -> Result<String, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::create_voice_clone(name, audio_samples, description)
        }

        fn design_voice(name: String, characteristics: VoiceDesignParams) -> Result<String, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::design_voice(name, characteristics)
        }

        fn convert_voice(
            input_audio: Vec<u8>,
            target_voice: crate::golem::tts::voices::Voice,
            preserve_timing: Option<bool>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::convert_voice(input_audio, target_voice.get_id(), preserve_timing)
        }

        fn generate_sound_effect(
            description: String,
            duration_seconds: Option<f32>,
            style_influence: Option<f32>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::generate_sound_effect(description, duration_seconds, style_influence)
        }

        fn create_lexicon(
            _name: String,
            _language: String,
            _entries: Option<Vec<crate::golem::tts::advanced::PronunciationEntry>>,
        ) -> Result<crate::golem::tts::advanced::PronunciationLexicon, TtsError> {
            Err(TtsError::UnsupportedOperation(
                "Lexicon management unsupported".to_string(),
            ))
        }

        fn synthesize_long_form(
            content: String,
            voice: crate::golem::tts::voices::Voice,
            output_location: String,
            chapter_breaks: Option<Vec<u32>>,
        ) -> Result<LongFormResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            Impl::synthesize_long_form(content, voice.get_id(), output_location, chapter_breaks)
        }
    }
}

#[cfg(feature = "durability")]
mod durable_impl {
    use super::*;
    use golem_rust::bindings::golem::durability::durability::DurableFunctionType;
    use golem_rust::durability::Durability;
    use golem_rust::{with_persistence_level, FromValueAndType, IntoValue, PersistenceLevel};
    use std::cell::RefCell;

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct VoicesInput {
        filter: Option<VoiceFilter>,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct VoiceInput {
        voice_id: String,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct SearchVoiceInput {
        query: String,
        filter: Option<VoiceFilter>,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct SynthesizeInput {
        input: TextInput,
        voice_id: String,
        options: Option<WitSynthesisOptions>,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct BatchInput {
        requests: Vec<SynthesizeInput>,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct TimingInput {
        input: TextInput,
        voice_id: String,
    }

    #[derive(Debug, Clone, PartialEq, IntoValue, FromValueAndType)]
    struct StreamInput {
        voice_id: String,
        options: Option<SynthesisOptions>,
    }

    impl From<&TtsError> for TtsError {
        fn from(err: &TtsError) -> Self {
            err.clone()
        }
    }

    impl<Impl: ExtendedGuest> VoicesGuest for DurableTts<Impl> {
        type Voice = crate::voices::VoiceResource;
        type VoiceResults = DurableVoiceResults;

        fn list_voices(
            filter: Option<VoiceFilter>,
        ) -> Result<crate::golem::tts::voices::VoiceResults, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<Vec<VoiceInfo>, TtsError>::new(
                "golem_tts",
                "list_voices",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::list_voices(filter.clone())
                });
                let result = durability.persist(VoicesInput { filter }, result)?;
                Ok(crate::golem::tts::voices::VoiceResults::new(
                    DurableVoiceResults::new(result),
                ))
            } else {
                let voices = durability.replay()?;
                Ok(crate::golem::tts::voices::VoiceResults::new(
                    DurableVoiceResults::new(voices),
                ))
            }
        }

        fn get_voice(voice_id: String) -> Result<crate::golem::tts::voices::Voice, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<VoiceInfo, TtsError>::new(
                "golem_tts",
                "get_voice",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::get_voice(voice_id.clone())
                });
                let voice = durability.persist(VoiceInput { voice_id }, result)?;
                Ok(crate::golem::tts::voices::Voice::new(
                    crate::voices::VoiceResource::from_info(&voice),
                ))
            } else {
                let voice = durability.replay()?;
                Ok(crate::golem::tts::voices::Voice::new(
                    crate::voices::VoiceResource::from_info(&voice),
                ))
            }
        }

        fn search_voices(
            query: String,
            filter: Option<VoiceFilter>,
        ) -> Result<crate::golem::tts::voices::VoiceResults, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<Vec<VoiceInfo>, TtsError>::new(
                "golem_tts",
                "search_voices",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::search_voices(query.clone(), filter.clone())
                });
                let result = durability.persist(SearchVoiceInput { query, filter }, result)?;
                Ok(crate::golem::tts::voices::VoiceResults::new(
                    DurableVoiceResults::new(result),
                ))
            } else {
                let voices = durability.replay()?;
                Ok(crate::golem::tts::voices::VoiceResults::new(
                    DurableVoiceResults::new(voices),
                ))
            }
        }

        fn list_languages() -> Result<Vec<String>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<Vec<String>, TtsError>::new(
                "golem_tts",
                "list_languages",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::list_languages()
                });
                durability.persist((), result)
            } else {
                durability.replay()
            }
        }
    }

    impl<Impl: ExtendedGuest> SynthesisGuest for DurableTts<Impl> {
        fn synthesize(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
            options: Option<WitSynthesisOptions>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<SynthesisResult, TtsError>::new(
                "golem_tts",
                "synthesize",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::synthesize(SynthesisRequest {
                        input: input.clone(),
                        voice_id: voice_id.clone(),
                        options: options.clone(),
                    })
                });
                durability.persist(
                    SynthesizeInput {
                        input,
                        voice_id,
                        options,
                    },
                    result,
                )
            } else {
                durability.replay()
            }
        }

        fn synthesize_batch(
            inputs: Vec<TextInput>,
            voice: crate::golem::tts::voices::Voice,
            options: Option<WitSynthesisOptions>,
        ) -> Result<Vec<SynthesisResult>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<Vec<SynthesisResult>, TtsError>::new(
                "golem_tts",
                "synthesize_batch",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            let requests: Vec<SynthesizeInput> = inputs
                .into_iter()
                .map(|input| SynthesizeInput {
                    input,
                    voice_id: voice_id.clone(),
                    options: options.clone(),
                })
                .collect();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::synthesize_batch(
                        requests
                            .iter()
                            .map(|req| SynthesisRequest {
                                input: req.input.clone(),
                                voice_id: req.voice_id.clone(),
                                options: req.options.clone(),
                            })
                            .collect(),
                    )
                });
                durability.persist(BatchInput { requests }, result)
            } else {
                durability.replay()
            }
        }

        fn get_timing_marks(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
        ) -> Result<Vec<TimingInfo>, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<Vec<TimingInfo>, TtsError>::new(
                "golem_tts",
                "get_timing_marks",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::get_timing_marks(input.clone(), voice_id.clone())
                });
                durability.persist(TimingInput { input, voice_id }, result)
            } else {
                durability.replay()
            }
        }

        fn validate_input(
            input: TextInput,
            voice: crate::golem::tts::voices::Voice,
        ) -> Result<ValidationResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<ValidationResult, TtsError>::new(
                "golem_tts",
                "validate_input",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::validate_input(input.clone(), voice_id.clone())
                });
                durability.persist(TimingInput { input, voice_id }, result)
            } else {
                durability.replay()
            }
        }
    }

    enum DurableStreamState<Impl: ExtendedGuest> {
        Live { stream: Impl::SynthesisStream },
        Replay,
    }

    pub struct DurableSynthesisStream<Impl: ExtendedGuest> {
        state: RefCell<DurableStreamState<Impl>>,
    }

    impl<Impl: ExtendedGuest> DurableSynthesisStream<Impl> {
        fn live(stream: Impl::SynthesisStream) -> Self {
            Self {
                state: RefCell::new(DurableStreamState::Live { stream }),
            }
        }
    }

    impl<Impl: ExtendedGuest> GuestSynthesisStream for DurableSynthesisStream<Impl> {
        fn send_text(&self, input: TextInput) -> Result<(), TtsError> {
            let durability = Durability::<(), TtsError>::new(
                "golem_tts",
                "stream_send_text",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let mut state = self.state.borrow_mut();
                let result = match &mut *state {
                    DurableStreamState::Live { stream } => {
                        with_persistence_level(PersistenceLevel::PersistNothing, || {
                            stream.send_text(input.clone())
                        })
                    }
                    DurableStreamState::Replay => Ok(()),
                };
                durability.persist(input, result)
            } else {
                durability.replay()
            }
        }

        fn finish(&self) -> Result<(), TtsError> {
            let durability = Durability::<(), TtsError>::new(
                "golem_tts",
                "stream_finish",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let mut state = self.state.borrow_mut();
                let result = match &mut *state {
                    DurableStreamState::Live { stream } => {
                        with_persistence_level(PersistenceLevel::PersistNothing, || stream.finish())
                    }
                    DurableStreamState::Replay => Ok(()),
                };
                durability.persist((), result)
            } else {
                durability.replay()
            }
        }

        fn receive_chunk(&self) -> Result<Option<AudioChunk>, TtsError> {
            let durability = Durability::<Option<AudioChunk>, TtsError>::new(
                "golem_tts",
                "stream_receive_chunk",
                DurableFunctionType::ReadRemote,
            );
            if durability.is_live() {
                let mut state = self.state.borrow_mut();
                let result = match &mut *state {
                    DurableStreamState::Live { stream } => {
                        with_persistence_level(PersistenceLevel::PersistNothing, || {
                            stream.receive_chunk()
                        })
                    }
                    DurableStreamState::Replay => Ok(None),
                };
                durability.persist((), result)
            } else {
                durability.replay()
            }
        }

        fn has_pending_audio(&self) -> bool {
            let state = self.state.borrow();
            match &*state {
                DurableStreamState::Live { stream } => stream.has_pending_audio(),
                DurableStreamState::Replay => false,
            }
        }

        fn get_status(&self) -> StreamStatus {
            let state = self.state.borrow();
            match &*state {
                DurableStreamState::Live { stream } => stream.get_status(),
                DurableStreamState::Replay => StreamStatus::Finished,
            }
        }

        fn close(&self) {
            let mut state = self.state.borrow_mut();
            if let DurableStreamState::Live { stream } = &mut *state {
                with_persistence_level(PersistenceLevel::PersistNothing, || stream.close());
            }
        }
    }

    pub struct DurableVoiceConversionStream<Impl: ExtendedGuest> {
        inner: RefCell<Impl::VoiceConversionStream>,
    }

    impl<Impl: ExtendedGuest> DurableVoiceConversionStream<Impl> {
        fn new(stream: Impl::VoiceConversionStream) -> Self {
            Self {
                inner: RefCell::new(stream),
            }
        }
    }

    impl<Impl: ExtendedGuest> GuestVoiceConversionStream for DurableVoiceConversionStream<Impl> {
        fn send_audio(&self, audio_data: Vec<u8>) -> Result<(), TtsError> {
            let durability = Durability::<(), TtsError>::new(
                "golem_tts",
                "voice_conversion_send_audio",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let mut inner = self.inner.borrow_mut();
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    inner.send_audio(audio_data.clone())
                });
                durability.persist(audio_data, result)
            } else {
                durability.replay()
            }
        }

        fn receive_converted(&self) -> Result<Option<AudioChunk>, TtsError> {
            let durability = Durability::<Option<AudioChunk>, TtsError>::new(
                "golem_tts",
                "voice_conversion_receive",
                DurableFunctionType::ReadRemote,
            );
            if durability.is_live() {
                let mut inner = self.inner.borrow_mut();
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    inner.receive_converted()
                });
                durability.persist((), result)
            } else {
                durability.replay()
            }
        }

        fn finish(&self) -> Result<(), TtsError> {
            let durability = Durability::<(), TtsError>::new(
                "golem_tts",
                "voice_conversion_finish",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let mut inner = self.inner.borrow_mut();
                let result =
                    with_persistence_level(PersistenceLevel::PersistNothing, || inner.finish());
                durability.persist((), result)
            } else {
                durability.replay()
            }
        }

        fn close(&self) {
            let mut inner = self.inner.borrow_mut();
            with_persistence_level(PersistenceLevel::PersistNothing, || inner.close());
        }
    }

    impl<Impl: ExtendedGuest> StreamingGuest for DurableTts<Impl> {
        type SynthesisStream = DurableSynthesisStream<Impl>;
        type VoiceConversionStream = DurableVoiceConversionStream<Impl>;

        fn create_stream(
            voice: crate::golem::tts::voices::Voice,
            options: Option<SynthesisOptions>,
        ) -> Result<Self::SynthesisStream, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<StreamInput, TtsError>::new(
                "golem_tts",
                "create_stream",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::create_stream(StreamRequest {
                        voice_id: voice_id.clone(),
                        options: options.clone(),
                    })
                });
                match result {
                    Ok(stream) => {
                        let _ = durability.persist(
                            StreamInput {
                                voice_id,
                                options,
                            },
                            Ok(StreamInput {
                                voice_id: "".to_string(),
                                options: None,
                            }),
                        );
                        Ok(DurableSynthesisStream::live(stream))
                    }
                    Err(error) => {
                        let _ = durability.persist(
                            StreamInput {
                                voice_id,
                                options,
                            },
                            Err(error.clone()),
                        );
                        Err(error)
                    }
                }
            } else {
                let _ = durability.replay::<StreamInput, TtsError>()?;
                Err(TtsError::UnsupportedOperation(
                    "Streaming replay not supported".to_string(),
                ))
            }
        }

        fn create_voice_conversion_stream(
            target_voice: crate::golem::tts::voices::Voice,
            options: Option<SynthesisOptions>,
        ) -> Result<Self::VoiceConversionStream, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<StreamInput, TtsError>::new(
                "golem_tts",
                "create_voice_conversion_stream",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = target_voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::create_voice_conversion_stream(StreamRequest {
                        voice_id: voice_id.clone(),
                        options: options.clone(),
                    })
                });
                match result {
                    Ok(stream) => {
                        let _ = durability.persist(
                            StreamInput {
                                voice_id,
                                options,
                            },
                            Ok(StreamInput {
                                voice_id: "".to_string(),
                                options: None,
                            }),
                        );
                        Ok(DurableVoiceConversionStream::new(stream))
                    }
                    Err(error) => {
                        let _ = durability.persist(
                            StreamInput {
                                voice_id,
                                options,
                            },
                            Err(error.clone()),
                        );
                        Err(error)
                    }
                }
            } else {
                let _ = durability.replay::<StreamInput, TtsError>()?;
                Err(TtsError::UnsupportedOperation(
                    "Voice conversion replay not supported".to_string(),
                ))
            }
        }
    }

    impl<Impl: ExtendedGuest> crate::golem::tts::advanced::Guest for DurableTts<Impl> {
        fn create_voice_clone(
            name: String,
            audio_samples: Vec<AudioSample>,
            description: Option<String>,
        ) -> Result<String, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<String, TtsError>::new(
                "golem_tts",
                "create_voice_clone",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::create_voice_clone(name.clone(), audio_samples.clone(), description.clone())
                });
                durability.persist((name, audio_samples, description), result)
            } else {
                durability.replay()
            }
        }

        fn design_voice(name: String, characteristics: VoiceDesignParams) -> Result<String, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<String, TtsError>::new(
                "golem_tts",
                "design_voice",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::design_voice(name.clone(), characteristics.clone())
                });
                durability.persist((name, characteristics), result)
            } else {
                durability.replay()
            }
        }

        fn convert_voice(
            input_audio: Vec<u8>,
            target_voice: crate::golem::tts::voices::Voice,
            preserve_timing: Option<bool>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<SynthesisResult, TtsError>::new(
                "golem_tts",
                "convert_voice",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = target_voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::convert_voice(input_audio.clone(), voice_id.clone(), preserve_timing)
                });
                durability.persist((input_audio, voice_id, preserve_timing), result)
            } else {
                durability.replay()
            }
        }

        fn generate_sound_effect(
            description: String,
            duration_seconds: Option<f32>,
            style_influence: Option<f32>,
        ) -> Result<SynthesisResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<SynthesisResult, TtsError>::new(
                "golem_tts",
                "generate_sound_effect",
                DurableFunctionType::WriteRemote,
            );
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::generate_sound_effect(
                        description.clone(),
                        duration_seconds,
                        style_influence,
                    )
                });
                durability.persist((description, duration_seconds, style_influence), result)
            } else {
                durability.replay()
            }
        }

        fn create_lexicon(
            _name: String,
            _language: String,
            _entries: Option<Vec<crate::golem::tts::advanced::PronunciationEntry>>,
        ) -> Result<crate::golem::tts::advanced::PronunciationLexicon, TtsError> {
            Err(TtsError::UnsupportedOperation(
                "Lexicon management unsupported".to_string(),
            ))
        }

        fn synthesize_long_form(
            content: String,
            voice: crate::golem::tts::voices::Voice,
            output_location: String,
            chapter_breaks: Option<Vec<u32>>,
        ) -> Result<LongFormResult, TtsError> {
            LOGGING_STATE.with_borrow_mut(|state| state.init());
            let durability = Durability::<LongFormResult, TtsError>::new(
                "golem_tts",
                "synthesize_long_form",
                DurableFunctionType::WriteRemote,
            );
            let voice_id = voice.get_id();
            if durability.is_live() {
                let result = with_persistence_level(PersistenceLevel::PersistNothing, || {
                    Impl::synthesize_long_form(
                        content.clone(),
                        voice_id.clone(),
                        output_location.clone(),
                        chapter_breaks.clone(),
                    )
                });
                durability.persist((content, voice_id, output_location, chapter_breaks), result)
            } else {
                durability.replay()
            }
        }
    }

        fn close(&self) {
            let mut inner = self.inner.borrow_mut();
            with_persistence_level(PersistenceLevel::PersistNothing, || inner.close());
        }
}
