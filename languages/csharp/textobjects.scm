(comment)+ @comment.around

(method_declaration
  body: (_ "{" (_)* @function.inside "}")) @function.around

(class_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

(struct_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

(record_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

(interface_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

(enum_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

(local_function_statement
  body: (_ "{" (_)* @function.inside "}")) @function.around

(lambda_expression
  body: (_ "{"? (_)* @function.inside "}"?)
) @function.around

(block) @block.around

(property_declaration
  body: (_ "{" (_)* @function.inside "}")) @function.around
