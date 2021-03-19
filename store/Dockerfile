FROM python:3
ADD src src
COPY access_tokens.json .

COPY requirements.txt .
RUN pip install -r requirements.txt  

CMD [ "python3", "./src/app.py" ]