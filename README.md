# SteamRec AI 
A website that feeds AI public steam profiles to make predictions on what it thinks would be the highest played/reviewed on said profile. Idea based on [Sprout](https://github.com/Ameobea/sprout) and data derived directly from Steam.

## How it works as a user
### 1. The user will be prompted to copy a link to their profile to input
![image of landing page](https://github.com/bossadapt/SteamRec-AI/assets/37967493/b8d7e344-86c9-4099-863d-c63e0968d132)
### 2. User pastes their profile into the input and hits the Search button
![image of loading the recommendations](https://github.com/bossadapt/SteamRec-AI/assets/37967493/dd6ea7bc-1802-4214-b139-ce9139fb91f2)
### 3. Games that were not a part of the person's reviews or games are displayed 5 at a time navigated with the '<Previous' and 'Next>' buttons
![image of the recommendations](https://github.com/bossadapt/SteamRec-AI/assets/37967493/8d912b39-2c59-4ca4-a017-75e9b0db48ae)

## How it works behind the scenes
The project is built with the following three coding languages Typescript, Rust, and Python. Each one of the languages has its role to play and is reachable as an API or webpage.

### Typescript in
Typescript acts as the user-facing interface with RegEx verifying links given and when finally given a valid link will reach out to the rust application via a fetch
### Rust in
The Rust portion does a lot of middleman work using a lot of my previous project's code 'Steam Crawler' as its base to collect information (scraped and steam API based) about the user that was placed into the search bar. The Rust portion then creates scorings of all games that were a part of the initial training and follows their order. It then sends out a request to the Python portion via Reqwest
### Python in/out
Python receives the list of floating scores and turns it into a tensor feeding the pre-trained simple PyTorch model with a custom loss function and returns what it expected all the scores to be. Python then returns the new predicted score list to the rust application that called it's Flask API

### Rust out
Rust then takes the returned score list from Python and combines it with the game metadata list created on the first startup. While combining the scores and the metadata, it also removes any overlapping games between the input list and the new score list. This new list is returned as a JSON object to the TSX application that calls its rocket.rs API

### Typescript out
The TSX portion receives the JSON and forces it to an object type with inbuilt error flags. The list is sorted and possible errors or faulty profiles are reported in the text under the search bar. Finally, the application sorts out all of the checkbox options and displays the 5 pages displayed at the start.
