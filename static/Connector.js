/**
 * 包装jquery的ajax请求，
 * @param {type} opts 
 * {
 *      url: "",
 *      params: {},
 *      success:function(json){},
 *      failure:function(json){}
 * }
 * @returns {void}
 */
Connector = function(opts){
    $.ajax({
        url:opts.url,
        data:JSON.stringify(opts.params),
        method:"POST",
        dataType: "json",
        contentType: "text/json",
        success:function(data){
            opts.success(data);
        },
        error:function(data){
            if(opts.failure && typeof opts.failure === "function") {
                opts.failure();
            } else {
                alert("网络错误，请检查您的网络连接!");
            }
        }
    });
};

