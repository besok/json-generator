### Json generator

 - [json standart](https://www.json.org/json-en.html)
 
### Modules
- input : file / folder / text
- output : text / file / folder / http 
- validator : json correctness / schema validness [later]
    - parser
- generator : data (by schema / by example) / schema from example [later]
    - types (primitive / objects)
    - works with schema  
    
#### Notes
- special  
    - correspondingly, the list of functions with conditions like:
        - string
            - randomFromList with list
            - randomFromFile with path
            - random with len
            - const with val
            - currentDate with format
            - currentDateTime with format
            - uuid
        - int: 
            - sequence with start pos
            - randomFromList with list
            - randomFromFile with path
            - const with val
            - random with start and end pos
        - array
            - array with func and element count 