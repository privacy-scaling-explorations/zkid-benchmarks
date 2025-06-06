import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.mopro.mopro_app.getFilePathFromAssets
import uniffi.mopro.GenerateProofResult
import uniffi.mopro.generateSha256Proof
import uniffi.mopro.verifySha256Proof

@Composable
fun FibonacciComponent() {
    var initTime by remember { mutableStateOf("init time:") }
    var provingTime by remember { mutableStateOf("proving time:") }
    var verifyingTime by remember { mutableStateOf("verifying time: ") }
    var valid by remember { mutableStateOf("valid:") }
    var output by remember { mutableStateOf("output:") }
    var res by remember {
        mutableStateOf<GenerateProofResult>(
            GenerateProofResult(proof = ByteArray(size = 0), inputs = ByteArray(size = 0))
        )
    }

    val inputs = mutableMapOf<String, List<String>>()
    inputs["a"] = listOf("0")
    inputs["b"] = listOf("1")

    val proverDataPath = getFilePathFromAssets("plonky2_fibonacci_pk.bin")
    val verifierDataPath = getFilePathFromAssets("plonky2_fibonacci_vk.bin")

    Box(modifier = Modifier.fillMaxSize().padding(16.dp), contentAlignment = Alignment.Center) {
        Button(
            onClick = {
                Thread(
                    Runnable {
                        val startTime = System.currentTimeMillis()
                        res = generateSha256Proof()
                        val endTime = System.currentTimeMillis()
                        provingTime = "proving time: " + (endTime - startTime).toString() + " ms"
                    }
                ).start()
            },
            modifier = Modifier.padding(top = 20.dp)
        ) { Text(text = "generate proof") }
        Button(
            onClick = {
                val startTime = System.currentTimeMillis()
                valid = "valid: " + verifySha256Proof(verifierDataPath, res.proof, res.inputs).toString()
                val endTime = System.currentTimeMillis()
                verifyingTime = "verifying time: " + (endTime - startTime).toString() + " ms"
                //output = "output: " + uniffi.mopro.toEthereumInputs(res.inputs)
            },
            modifier = Modifier.padding(top = 120.dp)
        ) { Text(text = "verify proof") }
        Text(
            text = "Multiplier proof",
            modifier = Modifier.padding(bottom = 180.dp),
            fontWeight = FontWeight.Bold
        )

        Text(text = initTime, modifier = Modifier.padding(top = 200.dp).width(200.dp))
        Text(text = provingTime, modifier = Modifier.padding(top = 250.dp).width(200.dp))
        Text(text = valid, modifier = Modifier.padding(top = 300.dp).width(200.dp))
        Text(text = verifyingTime, modifier = Modifier.padding(top = 350.dp).width(200.dp))
        //Text(text = output, modifier = Modifier.padding(top = 400.dp).width(200.dp))
    }
}