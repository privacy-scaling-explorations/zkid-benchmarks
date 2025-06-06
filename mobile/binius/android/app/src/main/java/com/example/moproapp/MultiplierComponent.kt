import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.motionEventSpy
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.example.moproapp.getFilePathFromAssets
import uniffi.mopro.CircomProofResult
import uniffi.mopro.CircomProof
import uniffi.mopro.G1
import uniffi.mopro.G2
import uniffi.mopro.generateCircomProof
import uniffi.mopro.verifyCircomProof
import uniffi.mopro.ProofLib
import uniffi.mopro.biniusSha256

@Composable
fun MultiplierComponent() {
    var prepareTime by remember { mutableStateOf("prepare time:") }
    var provingTime by remember { mutableStateOf("proving time:") }
    var verifyingTime by remember { mutableStateOf("verifying time: ") }

    Box(modifier = Modifier
        .fillMaxSize()
        .padding(16.dp), contentAlignment = Alignment.Center) {
        Button(
            onClick = {
                Thread(
                    Runnable {
                        val result = biniusSha256()
                        prepareTime = "prepare time: " + result.prepareTime + " ms\n"
                        provingTime = "proving time: " + result.proveTime + " ms\n"
                        verifyingTime = "verifying time: " + result.verifyTime + " ms\n"
                    }
                ).start()
            },
            modifier = Modifier.padding(top = 20.dp)
        ) { Text(text = "generate proof") }
        Text(
            text = "Binius Sha256 proof",
            modifier = Modifier.padding(bottom = 180.dp),
            fontWeight = FontWeight.Bold
        )

        Text(text = prepareTime, modifier = Modifier.padding(top = 250.dp).width(200.dp))
        Text(text = provingTime, modifier = Modifier
            .padding(top = 300.dp)
            .width(200.dp))
        Text(text = verifyingTime, modifier = Modifier
            .padding(top = 350.dp)
            .width(200.dp))
    }
}