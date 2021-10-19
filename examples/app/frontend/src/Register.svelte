<script>
    function register() {
        let name = window.prompt('Name?: '); 
        let email = window.prompt('Email?: ');
        let json = {
            name,
            email
        };
        let request = XMLHttpRequest();
        request.open('POST', 'http://localhost:3030/api/register');
        request.responseType = 'text';
        request.onload= function () {
            window.alert(request.response);
        };
        request.send();
    }
</script>

<button id="register" name="register" on:click="{register()}">Register</button>