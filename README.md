### Json generator

 - [json standart](https://www.json.org/json-en.html)
 
### modules
- input : (file.json / text with json / http request?) params with output format and number and 
- output : text with json / file.json / folder / http 
- validator : json correctness
- generator :  
    - types (primitive / objects)
    - works with schema  
    
#### generators
- examples  
    - correspondingly, the list of functions with conditions like:
        - string
            - randomFromList with list
            - randomFromFile with path
            - random with len
            - const with val
            - currentDate with format
            - currentDateTime with format
            - uuid
        - int 
            - sequence with start pos
            - randomFromList with list
            - randomFromFile with path
            - const with val
            - random with start and end pos
        - array
            - array with func and element count 
        - boolean
            - random
            - const with val     
            
- list :
    - sequence : usize
    - randomStr: usize - len of record
    - randomStrFromFile : path to the file , delimiter
    - randomIntFromFile : path to the file , delimiter
    - randomStrFromList : list of strings
    - randomIntFromList : list of integers
    - array : function , size
    - constStr : string
    - constInt : i64
    - rangeInt : start and stop usize
    - uuid 
    - currentDate : string with format
    - currentDateTame : string with format
    