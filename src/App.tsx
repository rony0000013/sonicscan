import {
  createEffect,
  createResource,
  createSignal,
  For,
  onMount,
  Show,
} from "solid-js";
import { A } from "@solidjs/router";
import { themeChange } from "theme-change";
import { invoke } from "@tauri-apps/api/core";
import { MediaRecorder, register } from "extendable-media-recorder";
import { connect } from "extendable-media-recorder-wav-encoder";
import { TrackResult } from "./schema";
import Fa from "solid-fa";
import {
  faMusic,
  faPause,
  faPlay,
  faPlus,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import { Motion, Presence } from "solid-motionone";
import "./Global.css";
import Logo from "./assets/logo.svg";
import CloudMusic from "./assets/cloud-music.svg";

import Card from "./card";

function App() {
  const [toast, setToast] = createSignal<string | null>(null);
  const [isRecording, setIsRecording] = createSignal(false);
  const [audioChunks, setAudioChunks] = createSignal<Blob[]>([]);
  const [addlist, setAddlist] = createSignal<TrackResult[]>([]);
  const [similarSongs, setSimilarSongs] = createSignal<TrackResult[]>([]);

  const changeToast = (message: string | null) => {
    if (toast() !== null) {
      setToast(null);
    }
    setToast(message);
    if (message !== null) {
      setTimeout(() => setToast(null), 5000);
    }
  };

  const fetchRecorder = async () => {
    await register(await connect());

    const stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        channelCount: 1,
        sampleRate: 44100,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
    });
    const audioContext = new AudioContext({ sampleRate: 44100 });
    const mediaStreamAudioSourceNode = new MediaStreamAudioSourceNode(
      audioContext,
      { mediaStream: stream },
    );
    const mediaStreamAudioDestinationNode = new MediaStreamAudioDestinationNode(
      audioContext,
    );

    mediaStreamAudioSourceNode.connect(mediaStreamAudioDestinationNode);

    const recorder = new MediaRecorder(mediaStreamAudioDestinationNode.stream, {
      mimeType: "audio/wav",
    });

    recorder.ondataavailable = (event) => {
      if (event.data && event.data.size > 0) {
        setAudioChunks((prev) => [...prev, event.data]);
      }
    };
    recorder.onstop = async () => {
      try {
        const audioBlob = new Blob(audioChunks(), { type: "audio/wav" });
        const bytes = await audioBlob.arrayBuffer();
        const audioArray = new Uint8Array(bytes);
        const similarSongs: TrackResult[] = await invoke(
          "similar_songs_command",
          {
            audio: Array.from(audioArray),
          },
        );
        setSimilarSongs(similarSongs);
        console.log(similarSongs);
        changeToast(`Similar Songs Found`);
        setAudioChunks([]);
      } catch (error) {
        console.error(error);
        changeToast(`Similar Song Fetch Error: ${error}`);
      }
    };
    return recorder;
  };
  const [recorder] = createResource(fetchRecorder);

  onMount(() => {
    try {
      invoke("ping_redis_command");
    } catch (error) {
      console.error(error);
    }
    themeChange();
  });

  createEffect(() => {
    if (isRecording()) {
      recorder()?.start();
    } else {
      recorder()?.stop();
    }
  });

  const add_music_to_lib = async (song: TrackResult) => {
    const is_present = await invoke("check_if_song_exists_command", {
      id: song.id,
    });
    if (is_present) {
      changeToast(`Song ${song.name} is already present in library`);
      return;
    }
    changeToast(`Adding song ${song.name} to library, please wait 1-2 minutes`);
    await invoke("add_music_to_db_command", {
      val: song,
    });
    changeToast(`Song ${song.name} added to library`);
    setAddlist((prev) => prev.filter((s) => s.id !== song.id));
  };

  const get_song_from_url = async (e: SubmitEvent) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget as HTMLFormElement);
    const url = formData.get("url");
    const regex = RegExp("youtu\.?be");

    try {
      if (regex.test(url?.toString() || "")) {
        try {
          await invoke("add_youtube_music_to_db_command", {
            url: url,
          });
          changeToast(`Added to database`);
          return;
        } catch (error) {
          console.error(error);
          changeToast(`Error: ${error}`);
        }
      }

      const list: any = await invoke(
        "get_song_from_url_command",
        {
          url: url,
        },
      );
      const list2: TrackResult[] = list;
      setAddlist(list2);
      changeToast(`Found ${list2.length} songs`);
    } catch (error) {
      console.error(error);
      changeToast(`Error: ${error}`);
    }
  };

  return (
    <div>
      <Presence>
        <Show when={toast()}>
          <Motion.div
            initial={{ opacity: 0, y: -50, scale: 0.5 }}
            animate={toast() !== null
              ? { opacity: 1, y: 0, scale: 1 }
              : { opacity: 0, y: -50, scale: 0.5 }}
            transition={{ duration: 0.3, easing: "ease-in-out" }}
            exit={{ opacity: 0, y: -50, scale: 0.5 }}
            class="toast toast-top toast-center z-100"
          >
            <div class="alert alert-info">
              <span>{toast()}</span>
            </div>
          </Motion.div>
        </Show>
      </Presence>

      <div class="flex flex-row sm:gap-4">
        <Motion.div
          hover={{
            scale: 1.1,
            transition: {
              duration: 0.3,
              easing: "ease-in-out",
            },
          }}
          class="flex-shrink mr-auto sm:ml-5"
        >
          <A
            href="/songs"
            class="btn btn-secondary my-5 ml-5 sm:ml-0 w-10 sm:w-40 "
          >
            <span class="hidden sm:inline">Song Library</span>
            <Fa icon={faMusic} />
          </A>
        </Motion.div>

        <div class="flex flex-grow flex-row items-center justify-center mx-auto">
          {
            /* <img
            src={logo}
            alt="Logo"
            class="h-15 object-contain rounded-2xl sm:mr-5"
          /> */
          }
          <Logo
            fill="currentColor"
            class="h-15 w-20 sm:h-20 sm:w-30 overflow-hidden text-info rounded-2xl "
          />
          <h1 class="text-2xl md:text-4xl text-center font-[Awesome] mt-5 text-accent">
            <span class="text-primary font-bold ">Sonic</span>Scan
          </h1>
        </div>

        <div class="flex-shrink justify-self-end sm:mr-5">
          <Motion.select
            // initial={{ opacity: 0 }}
            // animate={{ opacity: 1 }}
            // exit={{ opacity: 0 }}
            // transition={{ duration: 0.5, repeat: Infinity, easing: "ease-in-out" }}
            class="select select-bordered border-accent select-sm sm:select-md w-10 sm:w-auto max-w-sm my-5"
            data-choose-theme
          >
            <option value="">Select a theme ✨</option>
            <option value="light">Light ☀️</option>
            <option value="dark">Dark 🌃</option>
            <option value="cupcake">Cupcake 🎂</option>
            <option value="bumblebee">Bumblebee 🐝</option>
            <option value="emerald">Emerald 🌿</option>
            <option value="corporate">Corporate 🏢</option>
            <option value="synthwave">Synthwave 🎶</option>
            <option value="retro">Retro 🏙️</option>
            <option value="cyberpunk">Cyberpunk 🌐</option>
            <option value="valentine">Valentine 💌</option>
            <option value="halloween">Halloween 🎃</option>
            <option value="garden">Garden 🌼</option>
            <option value="forest">Forest 🌳</option>
            <option value="aqua">Aqua 🌊</option>
            <option value="lofi">Lofi 🎵</option>
            <option value="pastel">Pastel 🌸</option>
            <option value="fantasy">Fantasy 🌈</option>
            <option value="wireframe">Wireframe 📊</option>
            <option value="cmyk">Cmyk 🎨</option>
            <option value="autumn">Autumn 🍂</option>
            <option value="business">Business 🏦</option>
            <option value="acid">Acid 🌧️</option>
            <option value="lemonade">Lemonade 🍋</option>
            <option value="night">Night 🌃</option>
            <option value="coffee">Coffee ☕</option>
            <option value="winter">Winter 🌨️</option>
          </Motion.select>
        </div>
      </div>

      <Presence exitBeforeEnter>
        <Motion.div
          class={`min-h-64 md:min-h-96 flex items-center justify-center relative shadow-2xl bg-conic/decreasing from-violet-700 via-lime-300 to-violet-700 rounded-2xl backdrop-blur-lg`}
          initial={{ width: 0, height: 0, margin: "auto", opacity: 0 }}
          animate={{
            width: "100%",
            height: "100%",
            margin: "auto",
            opacity: 1,
          }}
          transition={{ duration: .5, easing: "ease-in" }}
        >
          <Show
            when={similarSongs().length == 0}
            fallback={
              <div class="flex flex-col justify-center my-5">
                <button
                  class="btn btn-error w-10 h-10 lg:w-15 lg:h-15 rounded-full mx-auto"
                  onClick={() => setSimilarSongs([])}
                >
                  <Fa icon={faXmark} size="2x" color="black" />
                </button>
                <div class="flex md:flex-row md:flex-wrap flex-col justify-center my-5 gap-5">
                  <For each={similarSongs()}>
                    {(song) => (
                      <Card
                        song={song}
                        add={false}
                        add_music_to_lib={add_music_to_lib}
                      />
                    )}
                  </For>
                </div>
              </div>
            }
          >
            <Motion.div
              class="absolute md:w-60 md:h-60 w-50 h-50 rounded-full flex items-center justify-center glass"
              initial={{ scale: 1 }}
              animate={isRecording()
                ? {
                  scale: [1, 1.2, 1],
                  transition: {
                    easing: "ease-in-out",
                    duration: 2,
                    repeat: Infinity,
                  },
                }
                : {
                  scale: 1,
                  transition: {
                    duration: 2,
                    easing: "ease-in-out",
                  },
                }}
            >
              <Motion.div
                class="absolute md:w-50 md:h-50 w-40 h-40 rounded-full flex items-center justify-center glass"
                initial={{ scale: 1 }}
                animate={isRecording()
                  ? {
                    scale: [1, 1.2, 1],
                    transition: {
                      delay: 0.25,
                      easing: "ease-in-out",
                      duration: 1.5,
                      repeat: Infinity,
                    },
                  }
                  : {
                    scale: 1,
                    transition: {
                      duration: 1.5,
                      easing: "ease-in-out",
                    },
                  }}
              >
                <Motion.div
                  class="absolute md:w-40 md:h-40 w-30 h-30 rounded-full flex items-center justify-center glass"
                  initial={{ scale: 1 }}
                  animate={isRecording()
                    ? {
                      scale: [1, 1.2, 1],
                      transition: {
                        delay: 0.5,
                        easing: "ease-in-out",
                        duration: 1,
                        repeat: Infinity,
                      },
                    }
                    : {
                      scale: 1,
                      transition: {
                        duration: 1,
                        easing: "ease-in-out",
                      },
                    }}
                >
                  <button
                    class="w-10 h-10 md:w-20 md:h-20 text-center flex items-center justify-center rounded-full glass"
                    onClick={() => setIsRecording((prev) => !prev)}
                  >
                    {isRecording()
                      ? <Fa icon={faPause} size="2x" color="black" spin />
                      : <Fa icon={faPlay} size="2x" color="black" />}
                  </button>
                </Motion.div>
              </Motion.div>
            </Motion.div>
          </Show>
        </Motion.div>
      </Presence>

      <Motion.h2
        initial={{ y: 0 }}
        animate={{ y: [0, 5, 0] }}
        transition={{ duration: 1.5, repeat: Infinity, easing: "ease" }}
        class="text-2xl md:text-4xl text-center my-5 mt-10 text-accent flex flex-col sm:flex-row flex-nowrap mx-auto justify-center items-center gap-2"
      >
        <span>Want to add Songs to</span>
        <div class="flex flex-row items-center gap-2">
          <span>Library</span>
          <CloudMusic
            fill="currentColor"
            class="w-7 h-7 md:w-10 md:h-10 text-primary rounded-4xl flex-shrink-0"
          />
          ?
        </div>
      </Motion.h2>

      <form
        class="form-control w-full mx-auto my-5 flex flex-col md:flex-row justify-center md:gap-5 items-center mb-5"
        onSubmit={get_song_from_url}
      >
        <label
          class="floating-label tooltip items-right justify-center"
          data-tip="It downloads song from jiosaavan as youtube and spotify download is not available or not working"
        >
          <span class="label-text">
            Add JioSaavn or YouTube or Spotify Link to add database
          </span>
          <Motion.input
            type="url"
            name="url"
            required
            placeholder="Add JioSaavn or YouTube or Spotify Link"
            class="input validator input-bordered hover:border-primary focus:border-accent mx-auto w-80 md:w-lg lg:w-3xl xl:w-4xl"
          />
          <p class="validator-hint">Must be valid URL</p>
        </label>
        <Motion.button
          hover={{
            scale: 1.1,
            rotate: 0,
            transition: {
              duration: 0.3,
              easing: "ease-in-out",
            },
          }}
          animate={{ rotate: [0, 10, 0, -10, 0] }}
          transition={{ duration: 5, easing: "ease-in-out", repeat: Infinity }}
          class="btn btn-secondary md:mb-6"
          type="submit"
        >
          Add <Fa icon={faPlus} />
        </Motion.button>
      </form>

      <div class="flex xs:mx-4 my-5 md:mx-0 h-full flex-col md:flex-row flex-wrap items-center justify-center gap-5">
        <For each={addlist()}>
          {(song, index) => (
            <Presence exitBeforeEnter>
              <Card
                data-index={index()}
                song={song}
                add_music_to_lib={add_music_to_lib}
                add={true}
              />
            </Presence>
          )}
        </For>
      </div>
    </div>
  );
}

export default App;
