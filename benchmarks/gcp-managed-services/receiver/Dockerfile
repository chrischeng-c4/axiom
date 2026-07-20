FROM python:3.13-slim
WORKDIR /app
COPY server.py /app/server.py
USER 65532:65532
ENV PORT=8080
CMD ["python", "/app/server.py"]
