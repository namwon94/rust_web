// 이후 API 요청 시 토큰을 헤더에 포함하는 예시
async function loadUserInfo() {
    const token = localStorage.getItem('access_token');
    
    const response = await fetch('/api/user/info', {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        }
    });
    
    if (response.status === 401) {
        // 토큰 만료 시 refresh 로직 또는 재로그인
        // refreshAccessToken() 같은 함수 호출
    }
    
    //주기적 갱신이 필요한 경우
    //return await response.json();
}

loadUserInfo();