#[cfg(test)]
mod auth_tests{
    use crate::auth::{create_jwt, validate_jwt};

    #[test]
    fn test_create_and_validate_jwt(){
        let jwt_result = create_jwt(1, 
                                                              "mymail@g.com".to_string(), 
                                                              "username".to_string());
        assert!(jwt_result.is_ok(), "Failed to create jwt {:?}", jwt_result);     
        let jwt = jwt_result.unwrap();
        let validation_result = validate_jwt(&jwt);
        assert!(validation_result.is_ok(), "Failed to validate JWT, {:?}", validation_result);
        let claims = validation_result.unwrap();
        assert!(claims.email == "mymail@g.com", "Email not equal");
        assert!(claims.username == "username", "Username not equal");
        assert!(claims.id == 1, "Identifier not equal");
        assert!(claims.exp - claims.iat > 0, "JWT expired");
    }

    #[test]
    fn test_create_and_validate_jwt_invalid(){
        let jwt_result = create_jwt(1, 
                                                              "mymail@g.com".to_string(), 
                                                              "username".to_string());
        assert!(jwt_result.is_ok(), "Failed to create jwt {:?}", jwt_result);     
        let jwt = jwt_result.unwrap();
        let mut jwt_chars: Vec<char> = jwt.chars().collect();
        jwt_chars[0] = 'a';
        let invalid_jwt: String = jwt_chars.into_iter().collect::<String>();
        assert!(validate_jwt(&invalid_jwt).is_err(), "Validation was successfull, but was expected to fail");
    }
}