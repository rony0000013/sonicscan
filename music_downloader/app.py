from yt_dlp import YoutubeDL
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import FileResponse
from starlette.background import BackgroundTask  # Import BackgroundTask
import tempfile
import os
import shutil
from pydub import AudioSegment

app = FastAPI()


# Define an async background task for cleanup
async def cleanup_temp_dir(temp_dir_path: str):
    """Cleans up the temporary directory after the response is sent."""
    print(f"Background task: Cleaning up temporary directory: {temp_dir_path}")
    try:
        shutil.rmtree(
            temp_dir_path, ignore_errors=True
        )  # Use ignore_errors for robustness
        print("Background task: Cleanup complete")
    except Exception as e:
        print(f"Background task: Error during cleanup of {temp_dir_path}: {e}")


@app.post("/youtube")
async def download_youtube_audio(req: Request):
    """
    Receives a YouTube URL, downloads the low-quality webm audio format,
    and returns the audio file as a response.
    """
    body = await req.json()
    url = body.get("url")

    if not url:
        raise HTTPException(status_code=400, detail="URL is invalid")

    temp_dir = (
        None  # Initialize temp_dir outside try block for cleanup if initial steps fail
    )

    try:
        # Step 1: Extract info to find the desired format ID
        ydl_opts_info = {
            "quiet": True,
            "simulate": True,
            "ignore_errors": False,
            "format": "ba",
        }
        print(f"Extracting info for URL: {url}")  # Debug print
        with YoutubeDL(ydl_opts_info) as ydl:
            try:
                info = ydl.extract_info(url, download=False)
                print("Info extraction successful")  # Debug print
            except Exception as e:
                print(f"yt-dlp info extraction failed for URL {url}: {e}")
                # It's good practice to include the original error in the detail for debugging
                raise HTTPException(
                    status_code=500, detail=f"Failed to extract video info: {e}"
                )

        all_formats = info.get("formats", [])
        all_formats.sort(key=lambda x: x.get("format_note", ""))
        audio_format_to_download = None
        for f in all_formats:
            is_audio_only = f.get("vcodec") == "none" and f.get("acodec") != "none"
            if is_audio_only and f.get("audio_ext") == "webm" and f.get("format_note") == "low":
                audio_format_to_download = f
                break

        if not audio_format_to_download:
            print("No low quality webm audio format found")  # Debug print
            raise HTTPException(
                status_code=404,
                detail="No low quality webm audio format found for this video",
            )

        print(
            f"Found format: {audio_format_to_download.get('format_id')} - {audio_format_to_download.get('format')}"
        )

        # Step 2: Download the specific format to a temporary directory
        temp_dir = tempfile.mkdtemp()  # Create a temporary directory
        print(f"Downloading to temporary directory: {temp_dir}")

        # Use a simple template within the temp dir that ensures a .webm extension
        # This makes Pydub loading more predictable
        download_outtmpl = os.path.join(temp_dir, "%(id)s.%(ext)s")
        webm_filepath = None  # Variable to store the path to the downloaded webm

        def progress_hook(d):
            nonlocal webm_filepath
            if d["status"] == "finished":
                # The 'filepath' key exists after the download is finished
                webm_filepath = d.get("filepath")
                print(f"Download finished. Webm filepath: {webm_filepath}")

        ydl_opts_download = {
            "format": audio_format_to_download["format_id"],
            "outtmpl": download_outtmpl,  # Use the template with temp_dir included
            "writedescription": False,
            "writeinfojson": False,
            "writethumbnail": False,
            "skip_download": False,
            "progress_hooks": [progress_hook],
            "quiet": True,
            "merge_output_format": None,  # Ensure no merging happens
            "postprocessors": [],  # No yt-dlp postprocessing needed as we'll use Pydub
            "ignore_errors": False,
        }

        print(
            f"Starting download for format_id {audio_format_to_download['format_id']}..."
        )
        with YoutubeDL(ydl_opts_download) as ydl:
            error_code = ydl.download([url])
            if error_code != 0:
                print(f"yt-dlp download failed with error code {error_code}")
                raise Exception(f"yt-dlp download failed with error code {error_code}")

        # Check if the download completed and the webm file exists
        if not webm_filepath or not os.path.exists(webm_filepath):
            print(
                "Error: Download process completed but webm filepath was not captured or does not exist."
            )
            # Attempt to find the file in the temp dir as a fallback
            temp_files = os.listdir(temp_dir)
            # Find the first file with .webm extension in the temp dir
            webm_files = [f for f in temp_files if f.endswith(".webm")]
            if webm_files:
                webm_filepath = os.path.join(temp_dir, webm_files[0])
                print(f"Fallback: Found webm file in temp dir: {webm_filepath}")
            else:
                print("Fallback failed: No webm files found in temp dir.")
                raise HTTPException(
                    status_code=500,
                    detail="Downloaded webm file not found after process completion",
                )

        # Step 3: Convert Webm to MP3 using Pydub
        # Create the output path for the MP3 file in the same temp directory
        mp3_filepath = os.path.splitext(webm_filepath)[0] + ".mp3"
        print(f"Converting webm to mp3: {webm_filepath} -> {mp3_filepath}")

        try:
            # Load the webm file
            audio = AudioSegment.from_file(webm_filepath, format="webm")
            # Export to mp3 file with a sample rate of 44.1K
            # audio.set_frame_rate(44100)
            audio.set_sample_width(2)
            audio.export(mp3_filepath, format="mp3")
            print("Conversion successful")
        except Exception as e:
            print(f"Error during audio conversion: {e}")
            # Raise an HTTPException if conversion fails
            raise HTTPException(
                status_code=500, detail=f"Failed to convert audio to MP3: {e}"
            )

        # Check if the MP3 file was created
        if not os.path.exists(mp3_filepath):
            print("Error: MP3 file was not created after conversion.")
            raise HTTPException(
                status_code=500, detail="MP3 file not found after conversion process"
            )

        # Step 4: Return the MP3 file as a response, scheduling cleanup
        original_filename = info.get("title", "audio")
        # Use .mp3 extension for the response filename
        sanitized_filename = "".join(
            [c for c in original_filename if c.isalnum() or c in (" ", ".", "_")]
        ).rstrip(" ._")
        response_filename = f"{sanitized_filename}.mp3"

        print(
            f"Returning file: {mp3_filepath} with filename {response_filename} and media type audio/mpeg"
        )

        # Use FileResponse with a BackgroundTask for cleanup
        return FileResponse(
            path=mp3_filepath,
            media_type="audio/mpeg",  # Set the MIME type for MP3
            filename=response_filename,
            background=BackgroundTask(cleanup_temp_dir, temp_dir),  # Schedule cleanup
        )

    except HTTPException:
        # Re-raise HTTPExceptions so FastAPI handles them correctly
        raise
    except Exception as e:
        # Catch any other unexpected exceptions during info extraction, download, or conversion
        print(f"An unexpected error occurred: {e}")
        # Clean up temporary directory if it was created before the response could be returned
        if temp_dir and os.path.exists(temp_dir):
            print(f"Cleaning up temp dir {temp_dir} due to an error.")
            # Use ignore_errors=True here as well
            shutil.rmtree(temp_dir, ignore_errors=True)
        # Return a generic 500 error to the client
        raise HTTPException(
            status_code=500, detail=f"An internal server error occurred: {e}"
        )


@app.get("/youtube")
async def get_youtube_info(req: Request):
    body = await req.json()
    url = body.get("url")

    if not url:
        raise HTTPException(status_code=400, detail="URL is invalid")

    try:
        # Step 1: Extract info to find the desired format ID
        ydl_opts_info = {
            "quiet": True,
            "simulate": True,
        }
        with YoutubeDL(ydl_opts_info) as ydl:
            info = ydl.extract_info(url, download=False)
            return {
                "id": info.get("id", ""),
                "title": info.get("title", ""),
                "thumbnail": info.get("thumbnail", ""),
                "duration": info.get("duration", ""),
                "url": info.get("original_url", url),
                "uploader": info.get("uploader", ""),
                "channel_url": info.get("channel_url", ""),
                "description": info.get("description", ""),
                "timestamp": info.get("timestamp", ""),
                "upload_date": info.get("upload_date", "")
            }
    
    except HTTPException:
        # Re-raise HTTPExceptions so FastAPI handles them correctly
        raise
    except Exception as e:
        # Return a generic 500 error to the client
        raise HTTPException(
            status_code=500, detail=f"An internal server error occurred: {e}"
        )
        


# You would typically run this using uvicorn: uvicorn main:app --reload
# (assuming this code is in main.py)
