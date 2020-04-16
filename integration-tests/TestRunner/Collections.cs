using System.Collections.Generic;
using Xunit;

namespace TestRunner
{
    public class Collections
    {
        [Fact]
        public void ReturnVecInt()
        {
            var expected = IntegrationTests.ReturnVec();
            var actual = new List<int>() { 1, 2, 3, 4 };
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void ReturnVecIntRepeated()
        {
            for (var count = 0; count < 100_000; count += 1)
            {
                var expected = IntegrationTests.ReturnVec();
                var actual = new List<int>() { 1, 2, 3, 4 };
                Assert.Equal(expected, actual);
            }
        }
    }
}
