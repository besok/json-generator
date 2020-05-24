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
    - random_str: usize - len of record
    - random_int : start and stop usize  
    -. random_str_from_file : path to the file , delimiter
    -. random_int_from_file : path to the file , delimiter
    -. random_str_from_list : list of strings
    -. random_int_from_list : list of integers
    -. array : function , size
    - uuid 
    - current_date_time : string with format
    