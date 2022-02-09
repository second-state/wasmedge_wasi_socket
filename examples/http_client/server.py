# Install dependencies: python3 -m pip install "uvicorn[standard]" fastapi
# Run: uvicorn server:app --port 1234

from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

@app.get("/get")
def hello():
    return "Hello World"

class Item(BaseModel):
    field1: str
    field2: str

@app.post("/post")
def read_item(item: Item):
    return item