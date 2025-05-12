import { TrackResult } from "./schema";
import { Fa } from "solid-fa";
import { faPlus } from "@fortawesome/free-solid-svg-icons";
import { Motion } from "solid-motionone";
import { Show } from "solid-js";

function Card({
    song,
    add,
    add_music_to_lib,
}: {
    song: TrackResult;
    add: boolean;
    add_music_to_lib: (song: TrackResult) => Promise<void>;
}) {
    const image_src = song.image.length > 0
        ? song.image[song.image.length - 1].url
        : "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b6/12in-Vinyl-LP-Record-Angle.jpg/500px-12in-Vinyl-LP-Record-Angle.jpg";
    const name = song.name || "Unknown";
    const artists = song.artists.primary.map((artist) => artist.name).join(
        ", ",
    );
    return (
        <Motion.div
            initial={{ opacity: 0.5, scale: 0.5, y: -50, rotate: "180deg" }}
            transition={{ duration: 0.4, easing: "ease-in-out" }}
            animate={{
                opacity: 1,
                scale: 1,
                y: 0,
                rotate: "0deg",
            }}
            exit={{
                opacity: 0.5,
                scale: 0.5,
                y: -50,
                transition: { duration: 0.4, easing: "ease-in-out" },
            }}
            class="card bg-base-100 min-h-50 rounded-lg max-w-80 min-w-64"
        >
            <figure class="absolute inset-0 rounded-lg ">
                <img
                    src={image_src}
                    alt={`${name} album cover`}
                    class="w-full h-full object-cover opacity-90 shadow-md"
                />
            </figure>
            <div class="card-body rounded-lg relative z-10 p-4 ">
                <div class="flex flex-row gap-5 items-center">
                    <div class="flex flex-grow flex-col text-center">
                        <h2 class="card-title text-primary backdrop-blur-md rounded-lg text-center">
                            {name}
                        </h2>
                        <p class="card-text text-secondary backdrop-blur-md rounded-lg">
                            {artists}
                        </p>
                    </div>
                    <Show when={add}>
                        <Motion.button
                            onClick={async () => await add_music_to_lib(song)}
                            animate={{ scale: [1, 1.1, 1] }}
                            transition={{ duration: 1, repeat: Infinity }}
                            class="btn btn-secondary border-black h-10 w-10 md:h-15 md:w-15 rounded-full flex-shrink"
                        >
                            <Fa
                                icon={faPlus}
                                class="text-lg md:text-2xl "
                            />
                        </Motion.button>
                    </Show>
                </div>
            </div>
            <div class="card-actions rounded-lg justify-end p-4">
                <Show when={song.downloadUrl.length > 0}>
                    <audio
                        controls
                        controls-list="nodownload"
                        src={song.downloadUrl[song.downloadUrl.length - 1]
                            .url}
                        class="h-10 md:h-20"
                    />
                </Show>
            </div>
        </Motion.div>
    );
}

export default Card;
