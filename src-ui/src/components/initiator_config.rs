use leptos::{
    component, create_node_ref, create_signal,
    html::Textarea, log, use_context, view, IntoView,
    NodeRef, Scope, SignalGet, SignalSet,
};

use crate::{
    app::{
        AppState, AppStateContext, LocalStreamRef,
        MediaStreamContext, RemoteStreamRef,
        RtcConnectionContext,
    },
    components::ConfigPanel,
    pages::MediaOption,
    rtc::{
        answer_offer, create_offer, init_media_stream,
        track_ice_candidate_event, track_local_stream,
        track_remote_stream,
    },
};

#[derive(Clone)]
enum ConfigState {
    ConfigPanel,
    LocalSDP,
    RemoteSDP,
}

#[component]
pub(crate) fn InitiatorConfig(
    cx: Scope,
    media_option: leptos::ReadSignal<MediaOption>,
    set_media_option: leptos::WriteSignal<MediaOption>,
) -> impl IntoView {
    let set_app_state =
        use_context::<AppStateContext>(cx).unwrap().1;
    let (config_state, set_config_state) =
        create_signal(cx, ConfigState::ConfigPanel);
    let rtc_pc =
        use_context::<RtcConnectionContext>(cx).unwrap().0;
    let set_media_stream =
        use_context::<MediaStreamContext>(cx).unwrap().1;
    let local_sdp_ref: NodeRef<Textarea> =
        create_node_ref(cx);
    let remote_sdp_ref: NodeRef<Textarea> =
        create_node_ref(cx);
    let local_stream_ref =
        use_context::<LocalStreamRef>(cx).unwrap().0;
    let remote_stream_ref =
        use_context::<RemoteStreamRef>(cx).unwrap().0;

    let on_generate_key = move |_| {
        match rtc_pc.get() {
            Some(pc) => {
                leptos::spawn_local(async move {
                    track_ice_candidate_event(
                        &pc,
                        rtc_pc,
                        local_sdp_ref,
                    ).expect("failed to track ice candidate event");

                    let media_stream = init_media_stream(
                        set_media_stream,
                        media_option,
                    )
                    .await
                    .expect("failed to init media stream");
                    track_local_stream(
                        &pc,
                        local_stream_ref,
                        media_stream,
                    )
                    .await
                    .expect("failed to track local stream");
                    track_remote_stream(
                        &pc,
                        remote_stream_ref,
                    )
                    .expect(
                        "failed to track remote stream",
                    );

                    if let Err(e) =
                        create_offer(&pc, local_sdp_ref)
                            .await
                    {
                        log!("error: {:?}", e);
                    };

                    set_config_state
                        .set(ConfigState::LocalSDP);
                });
            }
            None => {
                log!("error: no connection established.");
                set_app_state.set(AppState::Stable);
            }
        };
    };

    let on_remote_sdp_state = move |_| {
        set_config_state.set(ConfigState::RemoteSDP);
    };

    let on_connect = move |_| {
        match rtc_pc.get() {
            Some(pc) => {
                leptos::spawn_local(async move {
                    let remote_sdp_el =
                        remote_sdp_ref.get().unwrap();
                    let remote_sdp = remote_sdp_el.value();
                    // log!("remote_sdp: {:?}", remote_sdp);

                    if remote_sdp.is_empty() {
                        log!("Remote code is required.");
                        return;
                    }

                    if let Err(e) = answer_offer(
                        &remote_sdp,
                        &pc,
                        local_sdp_ref,
                    )
                    .await
                    {
                        log!("error: {:?}", e);
                    };

                    if let Some(el) = local_sdp_ref.get() {
                        el.set_value("");
                    };
                    if let Some(el) = remote_sdp_ref.get() {
                        el.set_value("");
                    };

                    set_config_state
                        .set(ConfigState::ConfigPanel);
                    set_app_state.set(AppState::Connected);
                });
            }
            None => {
                log!("error: no connection established.");
                set_app_state.set(AppState::Stable);
            }
        }
    };

    view! { cx,
            <div
                class="flex flex-col items-center h-full w-full"
                style:display=move || match config_state.get() {
                    ConfigState::ConfigPanel => "flex",
                    _ => "none",
                }
            >
                <ConfigPanel
                    set_media_option=set_media_option
                />

                <button
                    class="bg-blue-400 text-white rounded-lg p-2 mt-8 hover:bg-gray-600"
                    on:click=on_generate_key
                    >
                    "generate key"
                </button>
            </div>

            <div
                class="flex flex-col items-center h-[30%] w-full"
                style:display=move || match config_state.get() {
                    ConfigState::LocalSDP => "flex",
                    _ => "none",
                }
            >
                <div class="flex flex-col m-auto">
                    <label class="text-blue-700" for="local_sdp">"Local Key: "</label>
                    <textarea
                        node_ref=local_sdp_ref
                        class="border-blue-400 border-2 w-[40vw] h-[20vh] rounded-lg p-2"
                        type="text"
                        id="local_sdp"
                    />
                </div>

                <button
                    class="bg-blue-400 text-white rounded-lg p-2 mt-8 hover:bg-gray-600"
                    on:click=on_remote_sdp_state
                    >
                    "Next"
                </button>
            </div>

            <div
                class="flex flex-col items-center h-[30%] w-full"
                style:display=move || match config_state.get() {
                    ConfigState::RemoteSDP => "flex",
                    _ => "none",
                }
            >
                <div class="flex flex-col m-auto">
                    <label class="text-blue-700" for="local_sdp">"Remote Key: "</label>
                    <textarea
                        node_ref=remote_sdp_ref
                        class="border-blue-400 border-2 w-[40vw] h-[20vh] rounded-lg p-2"
                        type="text"
                        id="remote_sdp"
                    />
                </div>

                <button
                    class="bg-blue-400 text-white rounded-lg p-2 mt-8 hover:bg-gray-600"
                    on:click=on_connect
                    >
                    "Connect"
                </button>
            </div>
    }
}