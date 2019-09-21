function showAlert(warn) {
    let primary = document.getElementById("primary_form");
    let login = document.getElementById("form_login");
    let alert = document.createElement("div");
    alert.setAttribute("class", "alert alert-danger");
    alert.role = "alert";
    alert.innerHTML = warn;
    primary.insertBefore(alert, login);
}

function doLogin() {
    let username = document.getElementById("login-username").value;
    let password = document.getElementById("login-password").value;
    let type = document.getElementsByName("optradio");
    let typeVal = 0;
    if (type[0].checked) typeVal = 0;
    if (type[1].checked) typeVal = 1;
    if (type[2].checked) typeVal = 2;
    if (username == null || username === "") {
        showAlert("<b>Error :</b> Username cannot be empty");
        return;
    }
    if (password == null || password === "") {
        showAlert("<b>Error :</b> Password cannot be empty");
        return;
    }
    $.ajax({
        url: "/user/login",
        method: "post",
        contentType: "text/json",
        dataType: "json",
        data: JSON.stringify({name: username, passwd: sha256_digest(username + password), user_type: typeVal}),
        success: function (val) {
            const ret = val.result;
            if (ret === "success") {
                window.location = "/"
            } else {
                showAlert("<b>Error :</b> " + ret);
                document.getElementById("login-username").value = "";
                document.getElementById("login-password").value = "";
            }
        },
        error: function () {
            showAlert("<b>Error :</b> Network error !");
        }
    })
}
