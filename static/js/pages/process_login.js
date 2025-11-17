//로그인 시도
async function sessionLogin(event) {
    event.preventDefault();
    
    const email = document.getElementById('email').value;
    const password = document.getElementById('password').value;
    //const remember = document.getElementById('remember').checked;
    const errorMsg = document.getElementById('login-error');

    const formData = {
        email: email,
        password: password
    };
    //console.log('로그인 데이터:', formData);
    errorMsg.innerText = '';
    try{
        //실제 API호출
        const response = await fetch('/api/login_session', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(formData)
        });

        if(response.status === 401) {
            const errorText = await response.json();
            errorMsg.innerText = errorText.error;
            return;
        }

        if(!response.ok) {
            const errorText = await response.json();
            throw new Error(errorText.error || 'Network Error');
            return;
        }
        //서버에서 온 것: HTML구문 -> HTML문자열을 그대로 받음
        const html = await response.text();

        //HTML문자열을 DOM으로 변환하여 브라우저가 랜더링
        document.documentElement.innerHTML = html;
    }catch(error){
        console.error('Network Error : ', error);
        alert(error);
    }
}

/*
// 뒤로가기 시 토큰으로 다시 로드
window.addEventListener('popstate', async () => {
    const token = localStorage.getItem('access_token');
    if (token && window.location.pathname === '/success') {
        const response = await fetch('/success', {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        if (response.ok) {
            const html = await response.text();
            document.open();
            document.write(html);
            document.close();
        }
    }
});
*/