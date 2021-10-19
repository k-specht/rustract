<script>
    function register() {
        let name = window.prompt('Name?: '); 
        let email = window.prompt('Email?: ');
        let date = new Date()
        let json = {
            name: name,
            email: email,
            date: date.getFullYear() + '-' + (date.getMonth()+1) + '-' + date.getDate()
        };
        fetch('http://localhost:3030/api/register', {
            method: 'POST',
            body: JSON.stringify(json),
            headers: {
                "Content-Type": "application/json"
            }
        }).then(
            response => response.text()
        ).then(
            html => window.alert(html)
        );
    }
</script>

<button id="register" name="register" on:click="{register}">Register</button>