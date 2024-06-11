#takes the score list and shoves it into a trained model
from flask import Flask
from torch import nn
from markupsafe import escape
import torch

app = Flask(__name__)
classificationLen = 9800

class NeuralNetwork(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear_hardtanh_stack = nn.Sequential(
            nn.Linear(classificationLen, classificationLen),
            nn.Hardtanh(min_val=-3,max_val=12),
            nn.Linear(classificationLen, classificationLen),
            nn.Hardtanh(min_val=-3,max_val=12),
            nn.Linear(classificationLen, classificationLen),
        )
    def forward(self, x):
        logits = self.linear_hardtanh_stack(x)
        return logits
    
@app.route("/predict/<gamescores>")
def predict(gamescores):
    gamescores = str(gamescores)
    gamescoresFloat = torch.FloatTensor(list(map(lambda gs: float(gs), gamescores.split(","))))
    return model(gamescoresFloat).tolist()

if __name__ == '__main__':
    model = NeuralNetwork().to("cpu")
    model = torch.load('model2.pth',"cpu")
    app.run(host="0.0.0.0", port=5002, debug=True)