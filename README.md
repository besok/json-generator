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
    - sequence(usize) // starting point
    - random_str(usize) // len of record
    - random_int(usize, usize) // start and stop or delimiter can be omitted thus comma will be used
    - random_str_from_file(str,str) //  path to the file , delimiter or delimiter can be omitted thus comma will be used 
    - random_int_from_file(str,str) // path to the file , delimiter
    - random_str_from_list(str..) // list of strings
    - random_int_from_list(int..) // list of integers
    - array(func,usize) // function , size
    - uuid()
    - current_date_time(str) | current_date_time()  // string with format or empty

### Command line

```
json-generator.exe  -f "file path" -r 10  --pretty --print --to-folder folder--to-curl '-X POST ip'
```    

#### Arguments
| Short(-) | Long(--)  | Description                                                                                                 | Example                                                               |
|----------|-----------|-------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| b        | json-body | the text reprensting the json body                                                                          | --json-body \| -b '{"k":"v"}'                                         |
| f        | json-file | the path to file including the json                                                                         | --json-file \| -f c:\\folder\json.json                                |
| r        | repeat    | the number of repetetions of generating                                                                     | --repeat \| -r 10                                                     |
|          | pretty    | inserts formatting symbols in proper places                                                                 | --pretty                                                              |
|          | print     | prints logs                                                                                                 | --print                                                               |
|          | to-cmd    | show json in console(by default if outputs array is empty)                                                  | --to-cmd                                                              |
|          | to-file   | append generated jsons to file. If the file does not exist.  it creates a new one. The folder should exist. | --to-file c:\\folder\jsons.json                                       |
|          | to-folder | creates new files and place it to the selected folder.  It creates folder if it not exists.                 | --to-file c:\\folder                                                  |
|          | to-curl   | sends jsons to the server using curl for that. In fact,  the -d will be added.                              | --to-curl '-H "Content-Type:application/json" -X POST 127.0.0.1:7878' |
| -h       | --help    | infroamtion about commands                                                                                  | -h \| --help                                                          |
| -V       | --version | version                                                                                                     | -V \| --version                                                       |
 
 