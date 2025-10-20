//로그인 시도
async function handleLogin(event) {
    event.preventDefault();
    
    const email = document.getElementById('email').value;
    const password = document.getElementById('password').value;
    //const remember = document.getElementById('remember').checked;

    const formData = {
        email: email,
        password: password
    };
    console.log('로그인 데이터:', formData);

    try{
        //실제 API호출
        const response = await fetch('/api/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(formData)
        });

        if(!response.ok) {
            throw new Error('로그인 실패: ${response.status}');
        }

        //서버에서 온 것: HTML구문 -> HTML문자열을 그대로 받음
        const html = await response.text();

        //HTML문자열을 DOM으로 변환하여 브라우저가 랜더링
        document.documentElement.innerHTML = html;
    }catch{
        console.error('Error : ', error);
        alert('로그인 중 오류가 발생했습니다.');
    }
}