FROM ghcr.io/astral-sh/uv:python3.12-alpine

WORKDIR /app

COPY . .

# Install required system dependencies
RUN apk add --no-cache ffmpeg

# Install Python dependencies
RUN uv sync --no-cache

EXPOSE 8000

CMD ["uv", "run", "fastapi", "run", "app.py"]