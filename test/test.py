import requests
import unittest

url = 'http://localhost:8000'
view1 = "10.10.0.4:13801,10.10.0.5:13802"
view2 = "10.10.0.4:13800,10.10.0.5:13800,10.10.0.5:13800"
view2 = "10.10.0.4:13800,10.10.0.5:13800,10.10.0.5:13800,10.10.0.6:13800"

class load_balancer(unittest.TestCase):
    def test_view_change(self):
        response = requests.put(f'{url}/kvs/keys/view-change', json={'view':view1, 'repl_factor': 2})
        print(response.status_code)
        self.assertEqual(response.status_code, 200)
        print(response.json())
    
    def _test_delete(self):
        response = requests.delete('http://localhost:8000/kvs/jake', json={'value':'sawg'})
        print(response.status_code)
        self.assertEqual(response.status_code, 200)
        print(response.json())

# requests.delete('http://localhost:8000/kvs/jake', json={'value':'sawg'})
if __name__ == '__main__':
    unittest.main()