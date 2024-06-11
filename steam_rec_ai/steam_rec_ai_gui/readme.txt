Typescript(Next.js): 
-take user's steam url
-display games

Rust(using rocket.rs , reqwest and html parser:
(grab the extra info about the games being displayed on startup)
-check validity of steam url
-Check Visibility of games and reviews 
-grab games and reviews 
-filter out games not apart of classification list
-combine and convert convert games played + reviews to scores
-order them properly and ensure zeros are placed where needed
-call PYTHON feeding the list of 9800

Python(using flask an pytorch):
-convert the incoming data into a float tensor
-shove it into the model
-return

Python -> Rust -> Typescript