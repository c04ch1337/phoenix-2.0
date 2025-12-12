FROM python:3.11-slim

WORKDIR /app

COPY entrypoint.py /app/entrypoint.py

# Optional: expose a telemetry/health endpoint
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD python /app/entrypoint.py --healthcheck

ENTRYPOINT ["python", "/app/entrypoint.py"]

