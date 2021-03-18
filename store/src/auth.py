from public_keys import PUBLIC_KEYS
import rsa
import base64
from app import app
from flask import request
import random
import string
import json

class Auth():
    def __init__(self):
        self.public_keys = PUBLIC_KEYS
        self.access_tokens = []

    def create_access_token(self):
        letters = string.ascii_lowercase
        return ''.join(random.choice(letters) for i in range(10))

global auth

@app.before_first_request
def build_auth():
    global auth
    auth = Auth()


@app.route('/kvs/auth', methods=['POST'])
def authenticate():
    global auth
    user = request.get_json()
    if user.get('public_key') in auth.public_keys:
        if 'I am a trusted user' == rsa.decrypt(base64.b64decode(user.get('message').encode()), user.get('public_key')):
            auth.access_tokens.append(auth.create_access_token())
            return json.dumps({'access_token': auth.access_tokens[-1]}), 200
        return json.dumps({'message':'could not verify private key. Expecting encoded message \'I am a trusted user\''}), 403
    return json.dumps({'message':'not on list of authentificated hosts'}), 403
