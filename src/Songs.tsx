import { createEffect, createSignal, For, onMount, Show } from "solid-js";
import "./Global.css";
import { themeChange } from "theme-change";
import { invoke } from "@tauri-apps/api/core";
import { A } from "@solidjs/router";
import Fa from "solid-fa";
import { faHome, faRefresh, faTrash } from "@fortawesome/free-solid-svg-icons";
import { TrackResult } from "./schema";
import { Motion, Presence } from "solid-motionone";
import logo from "./assets/logo.png";

// import { Button } from "~/components/ui/button"

function Songs() {
    const [songs, setSongs] = createSignal<TrackResult[]>([]);
    const [toast, setToast] = createSignal<string | null>(null);

    const changeToast = (message: string | null) => {
        if (toast() !== null) {
            setToast(null);
        }
        setToast(message);
        if (message !== null) {
            setTimeout(() => setToast(null), 5000);
        }
    };

    onMount(() => {
        try {
            invoke("ping_redis_command");
        } catch (error) {
            console.error(error);
        }
        themeChange();
    });

    onMount(() => setTimeout(async () => await fetch_songs(), 2000));

    const fetch_songs = async () => {
        try {
            const songs: TrackResult[] = await invoke("get_all_songs_command");
            console.log(songs);
            changeToast(`Successfully fetched songs`);
            setSongs(songs);
        } catch (error) {
            // changeToast(error as string);
            changeToast(`Failed to get songs`);
            console.error(error);
        }
    };

    const delete_song = async (song: TrackResult) => {
        try {
            await invoke("delete_song_command", { id: song.id });
            setSongs(songs().filter((s) => s.id !== song.id));
            changeToast(`Successfully deleted song`);
        } catch (error) {
            // changeToast(error as string);
            changeToast(`Failed to delete song`);
            console.error(error);
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
                    class="flex-shrink mr-auto mx-5 "
                >
                    <A href="/" class="btn btn-secondary my-5 w-10 sm:w-30 ">
                        <span class="hidden sm:inline">Home</span>
                        <Fa icon={faHome} />
                    </A>
                </Motion.div>

                <div class="flex flex-grow flex-row items-center justify-center mx-auto">
                    <img
                        src={logo}
                        alt="Logo"
                        class="h-15 object-contain rounded-2xl sm:mr-5"
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
            <div class="flex flex-row justify-center my-5">
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
                    transition={{
                        duration: 5,
                        easing: "ease-in-out",
                        repeat: Infinity,
                    }}
                    class="btn btn-primary mx-auto"
                    onClick={fetch_songs}
                >
                    Refresh <Fa icon={faRefresh} />
                </Motion.button>
            </div>
            <table class="table table-xs table-pin-rows table-pin-cols overflow-x-auto">
                <thead>
                    <tr>
                        <th>Song Name</th>
                        <th>Artist</th>
                        <th>Album</th>
                        <th>Release Date</th>
                        <th>Duration</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <Show when={songs().length === 0}>
                        <tr>
                            <td colspan="6" class="text-center">
                                No songs found
                            </td>
                        </tr>
                    </Show>
                    <For each={songs()}>
                        {(song) => (
                            <tr>
                                <td>{song.name || "Unknown"}</td>
                                <td>
                                    {song.artists?.primary[0].name || "Unknown"}
                                </td>
                                <td>{song.album?.name || "Unknown"}</td>
                                <td>{song.releaseDate || "Unknown"}</td>
                                <td>{song.duration || "Unknown"}</td>
                                <td>
                                    <button
                                        class="btn btn-sm btn-primary"
                                        onClick={async () =>
                                            await delete_song(song)}
                                    >
                                        <Fa icon={faTrash} />
                                    </button>
                                </td>
                            </tr>
                        )}
                    </For>
                </tbody>
            </table>
        </div>
    );
}

export default Songs;
