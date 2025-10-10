;; Join up all the comments
(comment)+ @comment.around

;; Standard methods
(method_declaration
  body: (_ "{" (_)* @function.inside "}")) @function.around

;; Standard classes
(class_declaration
  body: (_ "{" (_)* @class.inside "}")) @class.around

;; Interface declarations
(method_declaration) @function.around

;; Lambda expressions
(lambda_expression
  body: (_ "{"? (_)* @function.inside "}"? )
) @function.around
