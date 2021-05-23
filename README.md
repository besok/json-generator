### Json generator
The simple console utility to generate JSON items according to the provided example composing JSON body 
and a set of functions that define the logic to generate new items.
 
The utility allows delivering the generated JSON to different sources such as an HTTP server, folder or file or console

### Overall

Given template:
```json
{
  "description": "the example how to create a json template to generate new jsons",
  "note": "the prefix | in a field name signals that the field carries a function to generate data.",
  "record": {
    "|type": "str_from_list(business,technical,analytical)", 
    "technical":{
       "|id": "uuid()",
       "|index": "seq()",
       "|update_tm": "dt()",
       "|created_tm": "dt(%Y-%m-%d %H:%M:%S)"
     },
    "|is_active": "bool()",
    "|name": "str(10,customer)",
    "|email": "str(5,,@gmail.com)",
    "|code": "str(5,,'(code)')",
    "|dsc": "str(20)",
    "geo": {
        "|country": "str_from_file(jsons/countries,,)",
        "|city": "str_from_file(jsons/cities,\n)",
        "|street": "str(10,,-street)",
        "|house": "int(1,1000)"
      },
    "|id_parent": "int_from_list(1,2,3,4,5,6,7)",
    "|related_records": "int(1,1000) -> array(5)"

  }
}
```

Generated json:
```json
{
  "description": "the example how to create a json template to generate new jsons",
  "note": "the prefix | in a field name signals that the field carries a function to generate data.",
  "record": {
    "code": "Upaz2(code)",
    "dsc": "gLgvDinPZg1aMu9LpPyp",
    "email": "PMWtc@gmail.com",
    "geo": {
      "city": "a",
      "country": "Australia",
      "house": 770,
      "street": "7Ke4CAHWpk-street"
    },
    "id_parent": 7,
    "is_active": false,
    "name": "customerXgKChm5t2b",
    "related_records": [
      263,
      489,
      390,
      226,
      361
    ],
    "technical": {
      "created_tm": "2021-05-23 13:09:27",
      "id": "339b0ca7-0e00-4d6e-8073-d270d7d56e2e",
      "index": 1,
      "update_tm": "2021-05-23 13:09:27"
    },
    "type": "analytical"
  }
}
```

### Generated rules
Overall, if the field does not have a specific prefix, depicting that the field carries a generator function, 
the value of the field will be taken and returned in the result.
Otherwise, if the field contains a prefix in its name the value is expected to be a string and describe the function to generate the values.
By default the prefix is ```|``` like in the example:
```json
{
  "|dynamic_field_with_generator": "seq()",
  "static_field": "constant"
}
``` 

*Note: the prefix sign can be changed if it interferes with the existing field into any other char.

### Generators
Every generator has a following syntax:
``` generator name ( arg1, arg2, ..) ```

If the generator contains another generator then the syntax obtain extra elements:
``` internal generator (arg1, arg2, .. ) -> encompassing generator (arg1, arg2, ..) ```

The generators can have empty arguments:
``` str(10,,postfix)```

The string literals can be placed as an argument straightly 
or encompassed by the single quotes:
``` str(10,literal,'with quotes')```
 

#### List of generators:
| Generator | Arguments | Description | Example |
|----------------------|--------------------------------------------|----------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------|
| seq | starting point,step | the sequentially-increase row of numbers (1,2,3,4 ...) | sequence(10)  |
| str | size of row | the row composed of random letters and numbers, predefined length | random_str(10) |
| int | low bound and high bound | the random number lying in predefined bounds | random_int(1,100) |
| random_str_from_file | path to file, delimiter(optional) | the list of string pulled off the predefined file note: delimiter can be omitted and the default delimiter(,) will be used | random_str_from_file(\home\user\json) random_str_from_file(\home\user\json,;) random_str_from_file(\home\user\json,\n) |
| random_int_from_file | path to file, delimiter(optional)  | list of numbers pulled off the predefined file note: delimiter can be omitted and the default delimiter(,) will be used  | random_int_from_file(c:\\user\json)  |
| random_str_from_list | list of values | the list of string | random_str_from_list(a,b,c,d) |
| random_int_from_list | list of values | list of numbers | random_int_from_list(1,2,3,4,5) |
| uuid |  | generated uuid  | uuid() |
| dt | format | the current date and time. By default can be ommited  and '%Y-%m-%d %H:%M:%S' will be used | currnet_date_time(%Y-%m-%d) |
| array | number of elements, generator for elements | the generator to get the array filled. | array(10,random_int(1,10)) |


### Command line example

```
json-generator.exe  -f "file path" -r 10  --pretty --print --to-folder folder --to-curl '-X POST ip'
```    

#### Command line Arguments
| Short | Long  | Description                                                                                                 | Example                                                               |
|----------|-----------|-------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| b        | json-body | the text reprensting the json body                                                                          | --json-body \| -b '{"k":"v"}'                                         |
| f        | json-file | the path to file including the json                                                                         | --json-file \| -f c:\\folder\json.json                                |
| r        | repeat    | the number of repetetions of generating                                                                     | --repeat \| -r 10                                                     |
|          | pretty    | inserts formatting symbols in proper places                                                                 | --pretty                                                              |
|          | print     | prints logs                                                                                                 | --print                                                               |
|          | to-cmd    | show json in console(by default if outputs array is empty)                                                  | --to-cmd                                                              |
|          | to-file   | append generated jsons to file. If the file does not exist.  it creates a new one. The folder should exist. | --to-file c:\\folder\jsons.json                                       |
|          | to-folder | creates new files and place it to the selected folder.  It creates folder if it not exists.                 | --to-file c:\\folder                                                  |
|          | to-curl   | sends jsons to the server using curl for that. In fact,  the -d will be added.                              | --to-curl "-H Content-Type:application/json -X POST 127.0.0.1:7878" |
| h       | help    | information  about commands                                                                                  | -h \| --help                                                          |
| V       | version | version                                                                                                     | -V \| --version                                                       |
 
**note**: for using --to-curl  parameter need to ensure the curl utility is installed.
#### Json example

```
{
  "person": {
    /* sequence(1) */
    "id": 1,
    /* current_date_time() */
    "update_tm": "",
    /*random_str(10)*/
    "name": "Eli\"za\"beth",
    /* random_str_from_list(a,b,c,d) */
    "surname": "EVa",
    /*random_int(20,40)*/
    "age": 10,
    /*array(3,sequence(1))*/
    "children_ids": [
      3,
      6
    ],
    "address": {
      /*random_str(10)*/
      "street": "Grip",
      /*random_int(1,100)*/
      "house": 10,
      /* random_str_from_file(C:\projects\json-generator\jsons\cities.txt,\r\n)*/
      "city": "Berlin"
    }
  }
}
```